//! Mock client implementation for use in testing.

use crate::client::sync::unbounded;
use crate::client::transport::router::SubscriptionRouter;
use crate::event::Event;
use crate::query::Query;
use crate::utils::uuid_str;
use crate::{Client, Error, Method, Request, Response, Result, Subscription, SubscriptionClient};
use async_trait::async_trait;
use std::collections::HashMap;

/// A mock client implementation for use in testing.
///
/// ## Examples
///
/// ```rust
/// use tendermint_rpc::{Client, Method, MockClient, MockRequestMatcher, MockRequestMethodMatcher};
///
/// const ABCI_INFO_RESPONSE: &str = r#"{
///   "jsonrpc": "2.0",
///   "id": "",
///   "result": {
///     "response": {
///       "data": "GaiaApp",
///       "last_block_height": "488120",
///       "last_block_app_hash": "2LnCw0fN+Zq/gs5SOuya/GRHUmtWftAqAkTUuoxl4g4="
///     }
///   }
/// }"#;
///
/// #[tokio::main]
/// async fn main() {
///     let matcher = MockRequestMethodMatcher::default()
///         .map(Method::AbciInfo, Ok(ABCI_INFO_RESPONSE.to_string()));
///     let mut client = MockClient::new(matcher);
///
///     let abci_info = client.abci_info().await.unwrap();
///     println!("Got mock ABCI info: {:?}", abci_info);
///     assert_eq!("GaiaApp".to_string(), abci_info.data);
/// }
/// ```
#[derive(Debug)]
pub struct MockClient<M: MockRequestMatcher> {
    matcher: M,
    router: SubscriptionRouter,
}

#[async_trait]
impl<M: MockRequestMatcher> Client for MockClient<M> {
    async fn perform<R>(&mut self, request: R) -> Result<R::Response>
    where
        R: Request,
    {
        self.matcher.response_for(request).ok_or_else(|| {
            Error::client_internal_error("no matching response for incoming request")
        })?
    }
}

impl<M: MockRequestMatcher> MockClient<M> {
    /// Create a new mock RPC client using the given request matcher.
    pub fn new(matcher: M) -> Self {
        Self {
            matcher,
            router: SubscriptionRouter::default(),
        }
    }

    /// Publishes the given event to all subscribers whose query exactly
    /// matches that of the event.
    pub async fn publish(&mut self, ev: &Event) {
        let _ = self.router.publish(ev).await;
    }
}

#[async_trait]
impl<M: MockRequestMatcher> SubscriptionClient for MockClient<M> {
    async fn subscribe(&mut self, query: Query) -> Result<Subscription> {
        let id = uuid_str();
        let (subs_tx, subs_rx) = unbounded();
        self.router.add(id.clone(), query.clone(), subs_tx);
        Ok(Subscription::new(id, query, subs_rx))
    }

    async fn unsubscribe(&mut self, query: Query) -> Result<()> {
        self.router.remove_by_query(query);
        Ok(())
    }
}

/// A trait required by the [`MockClient`] that allows for different approaches
/// to mocking responses for specific requests.
///
/// [`MockClient`]: struct.MockClient.html
pub trait MockRequestMatcher: Send + Sync {
    /// Provide the corresponding response for the given request (if any).
    fn response_for<R>(&self, request: R) -> Option<Result<R::Response>>
    where
        R: Request;
}

/// Provides a simple [`MockRequestMatcher`] implementation that simply maps
/// requests with specific methods to responses.
///
/// [`MockRequestMatcher`]: trait.MockRequestMatcher.html
#[derive(Debug)]
pub struct MockRequestMethodMatcher {
    mappings: HashMap<Method, Result<String>>,
}

impl MockRequestMatcher for MockRequestMethodMatcher {
    fn response_for<R>(&self, request: R) -> Option<Result<R::Response>>
    where
        R: Request,
    {
        self.mappings.get(&request.method()).map(|res| match res {
            Ok(json) => R::Response::from_string(json),
            Err(e) => Err(e.clone()),
        })
    }
}

impl Default for MockRequestMethodMatcher {
    fn default() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }
}

impl MockRequestMethodMatcher {
    /// Maps all incoming requests with the given method such that their
    /// corresponding response will be `response`.
    ///
    /// Successful responses must be JSON-encoded.
    #[allow(dead_code)]
    pub fn map(mut self, method: Method, response: Result<String>) -> Self {
        self.mappings.insert(method, response);
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::query::EventType;
    use futures::StreamExt;
    use std::path::PathBuf;
    use tendermint::block::Height;
    use tendermint::chain::Id;
    use tokio::fs;

    async fn read_json_fixture(name: &str) -> String {
        fs::read_to_string(PathBuf::from("./tests/support/").join(name.to_owned() + ".json"))
            .await
            .unwrap()
    }

    async fn read_event(name: &str) -> Event {
        Event::from_string(&read_json_fixture(name).await).unwrap()
    }

    #[tokio::test]
    async fn mock_client() {
        let abci_info_fixture = read_json_fixture("abci_info").await;
        let block_fixture = read_json_fixture("block").await;
        let matcher = MockRequestMethodMatcher::default()
            .map(Method::AbciInfo, Ok(abci_info_fixture))
            .map(Method::Block, Ok(block_fixture));
        let mut client = MockClient::new(matcher);

        let abci_info = client.abci_info().await.unwrap();
        assert_eq!("GaiaApp".to_string(), abci_info.data);
        assert_eq!(Height::from(488120_u32), abci_info.last_block_height);

        let block = client.block(Height::from(10_u32)).await.unwrap().block;
        assert_eq!(Height::from(10_u32), block.header.height);
        assert_eq!("cosmoshub-2".parse::<Id>().unwrap(), block.header.chain_id);
    }

    #[tokio::test]
    async fn mock_subscription_client() {
        let mut client = MockClient::new(MockRequestMethodMatcher::default());
        let event1 = read_event("event_new_block_1").await;
        let event2 = read_event("event_new_block_2").await;
        let event3 = read_event("event_new_block_3").await;
        let events = vec![event1, event2, event3];

        let subs1 = client.subscribe(EventType::NewBlock.into()).await.unwrap();
        let subs2 = client.subscribe(EventType::NewBlock.into()).await.unwrap();
        assert_ne!(subs1.id().to_string(), subs2.id().to_string());

        // We can do this because the underlying channels can buffer the
        // messages as we publish them.
        let subs1_events = subs1.take(3);
        let subs2_events = subs2.take(3);
        for ev in &events {
            client.publish(ev).await;
        }

        // Here each subscription's channel is drained.
        let subs1_events = subs1_events.collect::<Vec<Result<Event>>>().await;
        let subs2_events = subs2_events.collect::<Vec<Result<Event>>>().await;

        assert_eq!(3, subs1_events.len());
        assert_eq!(3, subs2_events.len());

        for i in 0..3 {
            assert!(events[i].eq(subs1_events[i].as_ref().unwrap()));
        }
    }
}

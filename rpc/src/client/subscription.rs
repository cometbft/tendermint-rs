//! Subscription- and subscription management-related functionality.

use crate::client::sync::{ChannelRx, ChannelTx};
use crate::event::Event;
use crate::query::Query;
use crate::Result;
use async_trait::async_trait;
use futures::task::{Context, Poll};
use futures::Stream;
use std::pin::Pin;

/// A client that exclusively provides [`Event`] subscription capabilities,
/// without any other RPC method support.
///
/// [`Event`]: event/struct.Event.html
#[async_trait]
pub trait SubscriptionClient {
    /// `/subscribe`: subscribe to receive events produced by the given query.
    async fn subscribe(&mut self, query: Query) -> Result<Subscription>;

    /// `/unsubscribe`: unsubscribe from events relating to the given query.
    ///
    /// This method is particularly useful when you want to terminate multiple
    /// [`Subscription`]s to the same [`Query`] simultaneously, or if you've
    /// joined multiple `Subscription`s together using [`select_all`] and you
    /// no longer have access to the individual `Subscription` instances to
    /// terminate them separately.
    ///
    /// [`Subscription`]: struct.Subscription.html
    /// [`Query`]: struct.Query.html
    /// [`select_all`]: https://docs.rs/futures/*/futures/stream/fn.select_all.html
    async fn unsubscribe(&mut self, query: Query) -> Result<()>;
}

pub(crate) type SubscriptionTx = ChannelTx<Result<Event>>;
pub(crate) type SubscriptionRx = ChannelRx<Result<Event>>;

/// An interface that can be used to asynchronously receive [`Event`]s for a
/// particular subscription.
///
/// ## Examples
///
/// ```
/// use tendermint_rpc::Subscription;
/// use futures::StreamExt;
///
/// /// Prints `count` events from the given subscription.
/// async fn print_events(subs: &mut Subscription, count: usize) {
///     let mut counter = 0_usize;
///     while let Some(res) = subs.next().await {
///         // Technically, a subscription produces `Result<Event, Error>`
///         // instances. Errors can be produced by the remote endpoint at any
///         // time and need to be handled here.
///         let ev = res.unwrap();
///         println!("Got incoming event: {:?}", ev);
///         counter += 1;
///         if counter >= count {
///             break
///         }
///     }
/// }
/// ```
///
/// [`Event`]: ./event/struct.Event.html
#[derive(Debug)]
pub struct Subscription {
    // A unique identifier for this subscription.
    id: String,
    // The query for which events will be produced.
    query: Query,
    // Our internal result event receiver for this subscription.
    rx: SubscriptionRx,
}

impl Stream for Subscription {
    type Item = Result<Event>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

impl Subscription {
    pub(crate) fn new(id: String, query: Query, rx: SubscriptionRx) -> Self {
        Self { id, query, rx }
    }

    /// Return this subscription's ID for informational purposes.
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn query(&self) -> &Query {
        &self.query
    }
}

//! Subscription- and subscription management-related functionality.

use core::pin::Pin;

use async_trait::async_trait;
use futures::{
    task::{Context, Poll},
    Stream,
};
use pin_project::pin_project;

use crate::{client::subscription::Inner, prelude::*, query::Query, v0_34::event::Event, Error};

/// A client that exclusively provides [`Event`] subscription capabilities,
/// without any other RPC method support.
#[async_trait]
pub trait SubscriptionClient {
    /// `/subscribe`: subscribe to receive events produced by the given query.
    async fn subscribe(&self, query: Query) -> Result<Subscription, Error>;

    /// `/unsubscribe`: unsubscribe from events relating to the given query.
    ///
    /// This method is particularly useful when you want to terminate multiple
    /// [`Subscription`]s to the same [`Query`] simultaneously, or if you've
    /// joined multiple `Subscription`s together using [`select_all`] and you
    /// no longer have access to the individual `Subscription` instances to
    /// terminate them separately.
    ///
    /// [`select_all`]: https://docs.rs/futures/*/futures/stream/fn.select_all.html
    async fn unsubscribe(&self, query: Query) -> Result<(), Error>;

    /// Subscription clients will usually have long-running underlying
    /// transports that will need to be closed at some point.
    fn close(self) -> Result<(), Error>;
}

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
#[pin_project]
#[derive(Debug)]
pub struct Subscription {
    #[pin]
    inner: Inner<Event>,
}

impl From<Inner<Event>> for Subscription {
    fn from(inner: Inner<Event>) -> Self {
        Self { inner }
    }
}

impl Stream for Subscription {
    type Item = Result<Event, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.project().rx.poll_next(cx)
    }
}

impl Subscription {
    /// Return this subscription's ID for informational purposes.
    pub fn id(&self) -> &str {
        &self.inner.id
    }

    pub fn query(&self) -> &Query {
        &self.inner.query
    }
}

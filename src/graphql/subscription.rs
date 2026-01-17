use async_graphql::futures_util::stream::{self, Stream};
use async_graphql::Subscription;

#[allow(dead_code)]
pub struct Subscriptions;

#[Subscription]
impl Subscriptions {
    async fn placeholder(&self) -> impl Stream<Item = bool> {
        stream::iter(vec![true])
    }
}

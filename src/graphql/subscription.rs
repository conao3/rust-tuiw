use async_graphql::Subscription;

pub struct Subscriptions;

#[Subscription]
impl Subscriptions {
    async fn placeholder(&self) -> bool {
        true
    }
}

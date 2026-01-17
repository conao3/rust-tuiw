use async_graphql::Object;

pub struct Query;

#[Object]
impl Query {
    async fn placeholder(&self) -> bool {
        true
    }
}

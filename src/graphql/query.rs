use async_graphql::Object;

#[allow(dead_code)]
pub struct Query;

#[Object]
impl Query {
    async fn placeholder(&self) -> bool {
        true
    }
}

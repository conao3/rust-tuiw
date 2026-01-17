use async_graphql::Object;

#[allow(dead_code)]
pub struct Mutation;

#[Object]
impl Mutation {
    async fn placeholder(&self) -> bool {
        true
    }
}

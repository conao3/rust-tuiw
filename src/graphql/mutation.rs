use async_graphql::Object;

pub struct Mutation;

#[Object]
impl Mutation {
    async fn placeholder(&self) -> bool {
        true
    }
}

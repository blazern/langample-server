use super::query::Query;
use crate::app_state::AppState;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub fn build_schema(app_state: AppState) -> AppSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(app_state)
        .finish()
}

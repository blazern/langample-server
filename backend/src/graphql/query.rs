use crate::app_state::AppState;
use crate::llm::chatgpt_lexical_items;
use crate::model::LexicalItemDetail;
use async_graphql::{Context, Error, ErrorExtensions, Object};

pub struct Query;

#[Object]
impl Query {
    async fn llm(
        &self,
        ctx: &Context<'_>,
        query: String,
        lang_from_iso3: String,
        lang_to_iso3: String,
    ) -> async_graphql::Result<Vec<LexicalItemDetail>> {
        let state = ctx.data::<AppState>()?;

        let query = query.trim();
        if query.is_empty() {
            return Err(Error::new("query must not be empty")
                .extend_with(|_, e| e.set("code", "BAD_USER_INPUT")));
        } else if MAX_QUERY_LEN < query.len() {
            return Err(
                Error::new(format!("query must not longer than {MAX_QUERY_LEN}"))
                    .extend_with(|_, e| e.set("code", "BAD_USER_INPUT")),
            );
        }
        if lang_from_iso3.len() != 3 || lang_to_iso3.len() != 3 {
            return Err(Error::new("languages must be ISO-3 (3 letters)")
                .extend_with(|_, e| e.set("code", "BAD_USER_INPUT")));
        }

        chatgpt_lexical_items::request(
            state.http_client(),
            state.chatgpt_key(),
            query,
            &lang_from_iso3,
            &lang_to_iso3,
            None,
            None,
        )
        .await
        .map_err(|(status, msg)| {
            Error::new("Upstream LLM error").extend_with(|_, e| {
                e.set("code", "UPSTREAM_LLM");
                e.set("httpStatus", status.as_u16());
                e.set("message", msg);
            })
        })
    }
}

const MAX_QUERY_LEN: usize = 50;

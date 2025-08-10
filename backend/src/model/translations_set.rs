use crate::model::sentence::Sentence;
use async_graphql::SimpleObject;

#[derive(SimpleObject, Clone, Debug, PartialEq, Eq)]
#[graphql(rename_fields = "camelCase")]
pub struct TranslationsSet {
    pub original: Sentence,
    pub translations: Vec<Sentence>,
}

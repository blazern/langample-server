use crate::model::sentence::Sentence;
use async_graphql::SimpleObject;

#[derive(SimpleObject, Clone, Debug, PartialEq, Eq)]
#[graphql(rename_fields = "camelCase")]
pub struct TranslationsSet {
    pub original: Sentence,
    pub translations: Vec<Sentence>,
    /// Possible values are 0-9
    pub translations_qualities: Option<Vec<i8>>,
}

use super::TranslationsSet;
use async_graphql::{SimpleObject, Union};

#[derive(SimpleObject, Clone, Debug, PartialEq, Eq)]
#[graphql(rename_fields = "camelCase")]
pub struct Forms {
    pub text: String,
    pub source: String,
}

#[derive(SimpleObject, Clone, Debug, PartialEq, Eq)]
#[graphql(rename_fields = "camelCase")]
pub struct WordTranslations {
    pub translations_set: TranslationsSet,
    pub source: String,
}

#[derive(SimpleObject, Clone, Debug, PartialEq, Eq)]
#[graphql(rename_fields = "camelCase")]
pub struct Synonyms {
    pub translations_set: TranslationsSet,
    pub source: String,
}

#[derive(SimpleObject, Clone, Debug, PartialEq, Eq)]
#[graphql(rename_fields = "camelCase")]
pub struct Explanation {
    pub text: String,
    pub source: String,
}

#[derive(SimpleObject, Clone, Debug, PartialEq, Eq)]
#[graphql(rename_fields = "camelCase")]
pub struct Example {
    pub translations_set: TranslationsSet,
    pub source: String,
}

#[derive(Union, Clone, Debug, PartialEq, Eq)]
pub enum LexicalItemDetail {
    Forms(Forms),
    WordTranslations(WordTranslations),
    Synonyms(Synonyms),
    Explanation(Explanation),
    Example(Example),
}

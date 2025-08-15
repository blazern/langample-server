use async_graphql::SimpleObject;

#[derive(SimpleObject, Clone, Debug, PartialEq, Eq)]
#[graphql(rename_fields = "camelCase")]
pub struct Sentence {
    pub text: String,
    pub lang_iso3: String,
    pub source: String,
}

impl Sentence {
    pub fn new(
        text: impl Into<String>,
        lang_iso3: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            text: text.into(),
            lang_iso3: lang_iso3.into(),
            source: source.into(),
        }
    }
}

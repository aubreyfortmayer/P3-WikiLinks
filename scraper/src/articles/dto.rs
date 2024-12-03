use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct PageLinksContinue {
    #[serde(rename = "gapcontinue")]
    pub gap_continue: Option<String>,

    #[serde(rename = "plcontinue")]
    pub pl_continue: Option<String>,

    #[serde(rename = "continue")]
    pub r#continue: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct PageLinksPage {
    pub title: String,
    pub links: Option<Vec<PageLink>>
}

#[derive(Deserialize, Debug)]
pub struct PageLink {
    pub title: String
}

#[derive(Deserialize, Debug)]
pub struct PageLinks {
    #[serde(rename = "continue")]
    pub r#continue: Option<PageLinksContinue>,
    pub query: Option<HashMap<String, Vec<PageLinksPage>>>
}

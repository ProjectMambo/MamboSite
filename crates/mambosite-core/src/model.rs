use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{MarkdownNode, SCHEMA_VERSION, SourceSpan};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PageStatus {
    #[default]
    Published,
    Draft,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mount {
    pub path: String,
    pub source: String,
}

/// Normalized author-controlled metadata before derived page fields are added.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    pub status: PageStatus,
    pub listed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<String>,
    pub mounts: Vec<Mount>,
    pub data: BTreeMap<String, Value>,
    /// Unknown compatibility values are retained instead of being lost.
    pub extra: BTreeMap<String, Value>,
}

impl Default for PageMetadata {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            slug: None,
            status: PageStatus::Published,
            listed: true,
            date: None,
            updated: None,
            tags: Vec::new(),
            aliases: Vec::new(),
            order: None,
            cover: None,
            mounts: Vec::new(),
            data: BTreeMap::new(),
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeadingRecord {
    pub id: String,
    pub level: u8,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockRecord {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<SourceSpan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReferenceSyntax {
    Markdown,
    Wiki,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ResolvedFragment {
    Heading { id: String },
    Block { id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ResolvedLinkTarget {
    Page {
        page_id: String,
        route: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        fragment: Option<ResolvedFragment>,
    },
    External {
        href: String,
    },
    /// Retained only when link validation is configured as non-strict.
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedLink {
    pub syntax: ReferenceSyntax,
    pub authored_destination: String,
    pub target: ResolvedLinkTarget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedEmbed {
    pub authored_destination: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option: Option<String>,
    pub instance_id: String,
    pub target: ResolvedLinkTarget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub schema_version: u32,
    pub id: String,
    pub route: String,
    pub source_path: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: PageStatus,
    pub listed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<String>,
    pub data: BTreeMap<String, Value>,
    pub extra: BTreeMap<String, Value>,
    pub headings: Vec<HeadingRecord>,
    pub blocks: Vec<BlockRecord>,
    pub directives: Vec<crate::ValidatedDirective>,
    pub body: MarkdownNode,
    pub children: Vec<String>,
    pub outgoing_links: Vec<ResolvedLink>,
    pub embeds: Vec<ResolvedEmbed>,
    pub backlinks: Vec<String>,
}

impl Page {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_parts(
        id: String,
        route: String,
        source_path: String,
        title: String,
        description: Option<String>,
        metadata: PageMetadata,
        headings: Vec<HeadingRecord>,
        blocks: Vec<BlockRecord>,
        directives: Vec<crate::ValidatedDirective>,
        body: MarkdownNode,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            id,
            route,
            source_path,
            title,
            description,
            status: metadata.status,
            listed: metadata.listed,
            date: metadata.date,
            updated: metadata.updated,
            tags: metadata.tags,
            aliases: metadata.aliases,
            order: metadata.order,
            cover: metadata.cover,
            data: metadata.data,
            extra: metadata.extra,
            headings,
            blocks,
            directives,
            body,
            children: Vec::new(),
            outgoing_links: Vec::new(),
            embeds: Vec::new(),
            backlinks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteMetadata {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub base_path: String,
    pub language: String,
    pub trailing_slash: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub schema_version: u32,
    pub site: SiteMetadata,
    pub entry_page: String,
    pub routes: BTreeMap<String, String>,
    pub pages: Vec<Page>,
}

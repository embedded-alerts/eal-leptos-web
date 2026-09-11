use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ApiHealth {
    pub(crate) service: String,
    pub(crate) status: String,
    pub(crate) environment: String,
    pub(crate) storage_mode: String,
    pub(crate) semantic_index_mode: String,
    pub(crate) production_ready: bool,
    pub(crate) database_connected: bool,
    pub(crate) supabase_configured: bool,
    pub(crate) embedding_mode: String,
    pub(crate) embedding_model: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DiscoveryMode {
    Seed,
    RobotsSitemap,
    Sitemap,
    Rss,
    LinkCrawl,
    ExternalIndex,
}

impl DiscoveryMode {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Seed => "seed",
            Self::RobotsSitemap => "robots sitemap",
            Self::Sitemap => "sitemap",
            Self::Rss => "RSS / Atom",
            Self::LinkCrawl => "same-domain links",
            Self::ExternalIndex => "external candidates",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SourceDomain {
    pub(crate) id: Uuid,
    pub(crate) tenant_id: Uuid,
    pub(crate) name: String,
    pub(crate) base_url: String,
    pub(crate) host: String,
    pub(crate) include_subdomains: bool,
    pub(crate) seed_urls: Vec<String>,
    pub(crate) discovery_modes: Vec<DiscoveryMode>,
    pub(crate) max_pages_per_scan: usize,
    pub(crate) source_priority: f32,
    pub(crate) respect_robots: bool,
    pub(crate) enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PageIndexRecord {
    pub(crate) id: Uuid,
    pub(crate) source_id: Uuid,
    pub(crate) previous_revision_id: Option<Uuid>,
    pub(crate) canonical_url: String,
    pub(crate) fetched_at: DateTime<Utc>,
    pub(crate) content_hash: String,
    pub(crate) title: Option<String>,
    pub(crate) summary: String,
    pub(crate) keywords: Vec<String>,
    pub(crate) entities: Vec<String>,
    pub(crate) model: serde_json::Value,
    pub(crate) extractor_version: String,
    pub(crate) segment_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MatchCandidate {
    pub(crate) id: Uuid,
    pub(crate) match_key: String,
    pub(crate) tenant_id: Uuid,
    pub(crate) alert_rule_id: Uuid,
    pub(crate) alert_rule_revision: u32,
    pub(crate) page_revision_id: Uuid,
    pub(crate) source_id: Uuid,
    pub(crate) canonical_url: String,
    pub(crate) content_hash: String,
    pub(crate) query_hash: String,
    pub(crate) model: serde_json::Value,
    pub(crate) score: f32,
    pub(crate) components: ScoreComponents,
    pub(crate) evidence: Vec<MatchEvidence>,
    pub(crate) state: String,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ScoreComponents {
    pub(crate) semantic: f32,
    pub(crate) lexical: f32,
    pub(crate) entity: f32,
    pub(crate) recency: f32,
    pub(crate) source_priority: f32,
    pub(crate) weights: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MatchEvidence {
    pub(crate) page_segment_kind: String,
    pub(crate) page_text: String,
    pub(crate) query_segment_kind: String,
    pub(crate) similarity: f32,
    pub(crate) weighted_similarity: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PreviewForm {
    pub(crate) query_text: String,
    pub(crate) threshold: f32,
    pub(crate) limit: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SemanticSearchRequest {
    pub(crate) query_text: String,
    pub(crate) source_ids: Vec<Uuid>,
    pub(crate) threshold: f32,
    pub(crate) limit: usize,
    pub(crate) cursor: Option<String>,
    pub(crate) expected_model: Option<serde_json::Value>,
    pub(crate) alert_rule: Option<serde_json::Value>,
}

impl TryFrom<PreviewForm> for SemanticSearchRequest {
    type Error = String;

    fn try_from(form: PreviewForm) -> Result<Self, Self::Error> {
        let query_text = form.query_text.trim().to_owned();
        if !(3..=2_000).contains(&query_text.chars().count()) {
            return Err("Interest must contain 3 to 2,000 characters.".into());
        }
        if !form.threshold.is_finite() || !(0.0..=1.0).contains(&form.threshold) {
            return Err("Threshold must be a finite number between 0 and 1.".into());
        }
        if !(1..=50).contains(&form.limit) {
            return Err("Preview limit must be between 1 and 50.".into());
        }
        Ok(Self {
            query_text,
            source_ids: Vec::new(),
            threshold: form.threshold,
            limit: form.limit,
            cursor: None,
            expected_model: None,
            alert_rule: None,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SemanticSearchResponse {
    pub(crate) query_text: String,
    pub(crate) model: serde_json::Value,
    pub(crate) results: Vec<SearchResult>,
    pub(crate) next_cursor: Option<String>,
    pub(crate) compared_pages: usize,
    pub(crate) skipped_cross_model_pages: usize,
    pub(crate) candidate_matches_created: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchResult {
    pub(crate) page_revision_id: Uuid,
    pub(crate) source_id: Uuid,
    pub(crate) canonical_url: String,
    pub(crate) title: Option<String>,
    pub(crate) summary: String,
    pub(crate) fetched_at: DateTime<Utc>,
    pub(crate) content_hash: String,
    pub(crate) model: serde_json::Value,
    pub(crate) score: f32,
    pub(crate) components: ScoreComponents,
    pub(crate) evidence: Vec<MatchEvidence>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previews_never_create_candidates() {
        let request = SemanticSearchRequest::try_from(PreviewForm {
            query_text: "Rust release changes affecting async runtimes".into(),
            threshold: 0.72,
            limit: 20,
        })
        .expect("valid preview");
        assert!(request.alert_rule.is_none());
    }

    #[test]
    fn preview_limits_are_bounded() {
        let error = SemanticSearchRequest::try_from(PreviewForm {
            query_text: "valid interest".into(),
            threshold: 0.72,
            limit: 51,
        })
        .expect_err("limit must be rejected");
        assert!(error.contains("50"));
    }
}

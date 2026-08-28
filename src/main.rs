mod api;
mod config;
mod models;

use std::sync::Arc;

use anyhow::Context;
use api::{ApiClient, ApiClientError};
use axum::{
    Form, Json, Router,
    extract::State,
    response::Html,
    routing::{get, post},
};
use config::{AppEnvironment, ConsoleConfig};
use leptos::prelude::*;
use models::{
    ApiHealth, MatchCandidate, MatchEvidence, PageIndexRecord, PreviewForm, ScoreComponents,
    SearchResult, SemanticSearchRequest, SemanticSearchResponse, SourceDomain,
};
use serde::Serialize;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    api: Arc<ApiClient>,
    environment: AppEnvironment,
    tenant_id: Uuid,
}

#[derive(Debug, Clone)]
struct DashboardData {
    health: Option<ApiHealth>,
    sources: Vec<SourceDomain>,
    pages: Vec<PageIndexRecord>,
    matches: Vec<MatchCandidate>,
    preview: Option<SemanticSearchResponse>,
    query_text: String,
    errors: Vec<String>,
    environment: String,
    tenant_label: String,
}

#[component]
fn Dashboard(data: DashboardData) -> impl IntoView {
    let DashboardData {
        health,
        sources,
        pages,
        matches,
        preview,
        query_text,
        errors,
        environment,
        tenant_label,
    } = data;

    let source_count = sources.len();
    let revision_count = pages.len();
    let candidate_count = matches.len();

    view! {
        <main class="shell">
            <header class="topbar">
                <div>
                    <p class="eyebrow">"EMBEDDED ALERTS / LEPTOS SSR"</p>
                    <h1>"Monitor ideas, not just keywords."</h1>
                </div>
                <div class="runtime">
                    <span>{environment}</span>
                    <code>{tenant_label}</code>
                </div>
            </header>

            <section class="hero">
                <p>
                    "This signed-in dashboard previews natural-language interests against locally fetched, "
                    "policy-approved page revisions. It never crawls arbitrary URLs and preview searches never create candidates or send notifications."
                </p>
                <div class="lock-note">
                    <strong>"Rule persistence and delivery are locked."</strong>
                    <span>
                        "Shared Auth, tenant-scoped PostgreSQL rules, and the DEN-3460 durable outbox remain production gates."
                    </span>
                </div>
            </section>

            <section class="stats" aria-label="Index summary">
                <article><strong>{source_count}</strong><span>"registered sources"</span></article>
                <article><strong>{revision_count}</strong><span>"page revisions"</span></article>
                <article><strong>{candidate_count}</strong><span>"candidate matches"</span></article>
                {health.map(health_card)}
            </section>

            {errors.into_iter().map(error_notice).collect_view()}

            <section class="panel preview-panel">
                <div class="panel-head">
                    <div>
                        <p class="section-number">"01 / INTEREST PREVIEW"</p>
                        <h2>"Describe what should find you"</h2>
                    </div>
                    <span class="pill">"read-only preview"</span>
                </div>
                <form method="post" action="/preview" class="preview-form">
                    <label>
                        <span>"Natural-language interest"</span>
                        <textarea name="query_text" rows="4" maxlength="2000" required>{query_text}</textarea>
                    </label>
                    <div class="form-row">
                        <label>
                            <span>"Similarity threshold"</span>
                            <input type="number" name="threshold" min="0" max="1" step="0.01" value="0.72" required/>
                        </label>
                        <label>
                            <span>"Preview limit"</span>
                            <input type="number" name="limit" min="1" max="50" value="12" required/>
                        </label>
                    </div>
                    <button type="submit">"Preview matches"</button>
                </form>
                {preview.map(preview_section)}
            </section>

            <section class="panel">
                <div class="panel-head">
                    <div>
                        <p class="section-number">"02 / SOURCE BOUNDARY"</p>
                        <h2>"Where discovery is allowed"</h2>
                    </div>
                    <a href="/" class="quiet-link">"refresh"</a>
                </div>
                {sources.is_empty().then(|| empty_state(
                    "No sources registered",
                    "An operator must register an exact public domain before any page can enter the index.",
                ))}
                <div class="source-grid">
                    {sources.into_iter().map(source_card).collect_view()}
                </div>
            </section>

            <section class="panel">
                <div class="panel-head">
                    <div>
                        <p class="section-number">"03 / RECENT REVISIONS"</p>
                        <h2>"What the owned index actually saw"</h2>
                    </div>
                    <span class="pill">"content-addressed"</span>
                </div>
                {pages.is_empty().then(|| empty_state(
                    "No page revisions",
                    "Run a bounded source scan from the Mash operations console.",
                ))}
                <div class="revision-list">
                    {pages.into_iter().take(12).map(page_card).collect_view()}
                </div>
            </section>

            <section class="panel">
                <div class="panel-head">
                    <div>
                        <p class="section-number">"04 / CANDIDATE INBOX"</p>
                        <h2>"Matches awaiting a delivery decision"</h2>
                    </div>
                    <span class="pill pill--locked">"delivery disabled"</span>
                </div>
                {matches.is_empty().then(|| empty_state(
                    "No candidates",
                    "Candidate creation requires an immutable alert-rule revision; preview searches intentionally omit it.",
                ))}
                <div class="candidate-list">
                    {matches.into_iter().take(20).map(candidate_card).collect_view()}
                </div>
            </section>
        </main>
    }
}

fn health_card(health: ApiHealth) -> impl IntoView {
    let detail = format!(
        "{} · {} · {} · db:{} · supabase:{} · production:{}",
        health.environment,
        health.storage_mode,
        health.semantic_index_mode,
        health.database_connected,
        health.supabase_configured,
        health.production_ready,
    );
    view! {
        <article class="health-card">
            <strong>{health.status}</strong>
            <span>{health.service}</span>
            <small>{detail}</small>
            <code>{format!("{} / {}", health.embedding_mode, health.embedding_model)}</code>
        </article>
    }
}

fn source_card(source: SourceDomain) -> impl IntoView {
    let class = if source.enabled {
        "source-card"
    } else {
        "source-card source-card--disabled"
    };
    let modes = source
        .discovery_modes
        .iter()
        .map(|mode| mode.label())
        .collect::<Vec<_>>()
        .join(" · ");
    let seeds = if source.seed_urls.is_empty() {
        "API default".to_owned()
    } else {
        source.seed_urls.join(" · ")
    };
    view! {
        <article class=class>
            <div class="card-head">
                <div>
                    <h3>{source.name}</h3>
                    <a href=source.base_url target="_blank" rel="noopener noreferrer">{source.host}</a>
                </div>
                <strong>{percent(source.source_priority)}</strong>
            </div>
            <p>{modes}</p>
            <dl>
                <div><dt>"source"</dt><dd><code>{short_uuid(source.id)}</code></dd></div>
                <div><dt>"tenant"</dt><dd><code>{short_uuid(source.tenant_id)}</code></dd></div>
                <div><dt>"budget"</dt><dd>{format!("{} pages", source.max_pages_per_scan)}</dd></div>
                <div><dt>"subdomains"</dt><dd>{yes_no(source.include_subdomains)}</dd></div>
                <div><dt>"robots"</dt><dd>{if source.respect_robots { "required" } else { "unsafe" }}</dd></div>
                <div><dt>"enabled"</dt><dd>{yes_no(source.enabled)}</dd></div>
            </dl>
            <small>{format!("Seeds: {seeds}")}</small>
        </article>
    }
}

fn page_card(page: PageIndexRecord) -> impl IntoView {
    let predecessor = page
        .previous_revision_id
        .map(short_uuid)
        .unwrap_or_else(|| "first revision".into());
    let keywords = page
        .keywords
        .iter()
        .take(8)
        .cloned()
        .collect::<Vec<_>>()
        .join(" · ");
    let entities = page
        .entities
        .iter()
        .take(6)
        .cloned()
        .collect::<Vec<_>>()
        .join(" · ");
    view! {
        <article class="revision-card">
            <div class="card-head">
                <div>
                    <p class="kicker">{format!("{} · source {}", short_uuid(page.id), short_uuid(page.source_id))}</p>
                    <h3>
                        <a href=page.canonical_url target="_blank" rel="noopener noreferrer">
                            {page.title.unwrap_or_else(|| "Untitled page".into())}
                        </a>
                    </h3>
                </div>
                <span class="pill">{format!("{} segments", page.segment_count)}</span>
            </div>
            <p>{page.summary}</p>
            <small>{keywords}</small>
            <small>{entities}</small>
            <dl>
                <div><dt>"fetched"</dt><dd>{timestamp(page.fetched_at)}</dd></div>
                <div><dt>"hash"</dt><dd><code>{short_hash(&page.content_hash)}</code></dd></div>
                <div><dt>"predecessor"</dt><dd><code>{predecessor}</code></dd></div>
                <div><dt>"extractor"</dt><dd><code>{page.extractor_version}</code></dd></div>
                <div><dt>"model"</dt><dd><code>{compact_json(&page.model)}</code></dd></div>
            </dl>
        </article>
    }
}

fn candidate_card(candidate: MatchCandidate) -> impl IntoView {
    view! {
        <article class="candidate-card">
            <div class="candidate-score">
                <strong>{percent(candidate.score)}</strong>
                <span>{candidate.state}</span>
            </div>
            <div>
                <p class="kicker">{format!("candidate {} · rule {}@{}", short_uuid(candidate.id), short_uuid(candidate.alert_rule_id), candidate.alert_rule_revision)}</p>
                <h3><a href=candidate.canonical_url target="_blank" rel="noopener noreferrer">"Open matched page"</a></h3>
                {score_breakdown(candidate.components)}
                {evidence_list(candidate.evidence)}
                <details>
                    <summary>"Identity and provenance"</summary>
                    <dl>
                        <div><dt>"tenant"</dt><dd><code>{candidate.tenant_id.to_string()}</code></dd></div>
                        <div><dt>"page revision"</dt><dd><code>{candidate.page_revision_id.to_string()}</code></dd></div>
                        <div><dt>"source"</dt><dd><code>{candidate.source_id.to_string()}</code></dd></div>
                        <div><dt>"match key"</dt><dd><code>{candidate.match_key}</code></dd></div>
                        <div><dt>"content hash"</dt><dd><code>{candidate.content_hash}</code></dd></div>
                        <div><dt>"query hash"</dt><dd><code>{candidate.query_hash}</code></dd></div>
                        <div><dt>"model"</dt><dd><code>{compact_json(&candidate.model)}</code></dd></div>
                        <div><dt>"created"</dt><dd>{timestamp(candidate.created_at)}</dd></div>
                    </dl>
                </details>
            </div>
        </article>
    }
}

fn preview_section(preview: SemanticSearchResponse) -> impl IntoView {
    let summary = format!(
        "{} pages compared · {} cross-model skipped · {} candidates created · model {}{}",
        preview.compared_pages,
        preview.skipped_cross_model_pages,
        preview.candidate_matches_created,
        compact_json(&preview.model),
        preview
            .next_cursor
            .as_ref()
            .map(|cursor| format!(" · next {cursor}"))
            .unwrap_or_default(),
    );
    view! {
        <section class="preview-results">
            <div class="preview-summary">
                <strong>{format!("{} result(s)", preview.results.len())}</strong>
                <span>{format!("Interest: {}", preview.query_text)}</span>
                <small>{summary}</small>
            </div>
            {preview.results.is_empty().then(|| empty_state(
                "No preview result crossed the threshold",
                "Try a broader interest or index additional policy-approved pages.",
            ))}
            <div class="result-list">
                {preview.results.into_iter().map(search_result_card).collect_view()}
            </div>
        </section>
    }
}

fn search_result_card(result: SearchResult) -> impl IntoView {
    view! {
        <article class="result-card">
            <div class="candidate-score"><strong>{percent(result.score)}</strong><span>"combined"</span></div>
            <div>
                <p class="kicker">{format!("page {} · source {}", short_uuid(result.page_revision_id), short_uuid(result.source_id))}</p>
                <h3><a href=result.canonical_url target="_blank" rel="noopener noreferrer">{result.title.unwrap_or_else(|| "Untitled page".into())}</a></h3>
                <p>{result.summary}</p>
                {score_breakdown(result.components)}
                {evidence_list(result.evidence)}
                <small>{format!("Fetched {} · hash {} · model {}", timestamp(result.fetched_at), short_hash(&result.content_hash), compact_json(&result.model))}</small>
            </div>
        </article>
    }
}

fn score_breakdown(score: ScoreComponents) -> impl IntoView {
    let weights = compact_json(&score.weights);
    view! {
        <div class="score-grid" title=weights>
            {score_cell("semantic", score.semantic)}
            {score_cell("lexical", score.lexical)}
            {score_cell("entity", score.entity)}
            {score_cell("recency", score.recency)}
            {score_cell("source", score.source_priority)}
        </div>
    }
}

fn score_cell(label: &'static str, value: f32) -> impl IntoView {
    view! {
        <div><span>{label}</span><strong>{percent(value)}</strong><meter min="0" max="1" value=format!("{value:.4}")>{percent(value)}</meter></div>
    }
}

fn evidence_list(evidence: Vec<MatchEvidence>) -> impl IntoView {
    view! {
        <details class="evidence">
            <summary>{format!("{} evidence segment(s)", evidence.len())}</summary>
            <ol>
                {evidence.into_iter().map(|item| view! {
                    <li>
                        <strong>{format!("{} ↔ {}", item.page_segment_kind, item.query_segment_kind)}</strong>
                        <span>{format!("{} weighted / {} raw", percent(item.weighted_similarity), percent(item.similarity))}</span>
                        <p>{item.page_text}</p>
                    </li>
                }).collect_view()}
            </ol>
        </details>
    }
}

fn error_notice(error: String) -> impl IntoView {
    view! { <div class="notice notice--error" role="status"><strong>"Dashboard boundary error"</strong><p>{error}</p></div> }
}

fn empty_state(title: &'static str, detail: &'static str) -> impl IntoView {
    view! { <div class="empty"><strong>{title}</strong><p>{detail}</p></div> }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = ConsoleConfig::from_env()?;
    let api = ApiClient::new(config.api_base_url.clone(), config.tenant_id)
        .context("configure Embedded Alerts API client")?;
    let state = AppState {
        api: Arc::new(api),
        environment: config.environment,
        tenant_id: config.tenant_id,
    };

    warn!(
        environment = state.environment.as_str(),
        tenant_context = "development_header",
        api_base_url = %config.api_base_url,
        "Leptos dashboard is preview-only and production startup is disabled"
    );

    let app = Router::new()
        .route("/", get(index))
        .route("/preview", post(preview))
        .route("/healthz", get(health))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind((config.host.as_str(), config.port))
        .await
        .with_context(|| format!("bind {}:{}", config.host, config.port))?;
    info!(address = %listener.local_addr()?, "Embedded Alerts Leptos dashboard listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn index(State(state): State<AppState>) -> Html<String> {
    render_page(load_dashboard(&state, String::new(), None, None).await)
}

async fn preview(State(state): State<AppState>, Form(form): Form<PreviewForm>) -> Html<String> {
    let query_text = form.query_text.trim().to_owned();
    match SemanticSearchRequest::try_from(form) {
        Ok(request) => render_page(load_dashboard(&state, query_text, Some(request), None).await),
        Err(error) => render_page(load_dashboard(&state, query_text, None, Some(error)).await),
    }
}

#[derive(Debug, Serialize)]
struct DashboardHealth {
    service: &'static str,
    status: &'static str,
    environment: &'static str,
    production_ready: bool,
    tenant_context: &'static str,
    api_reachable: bool,
    api_production_ready: Option<bool>,
}

async fn health(State(state): State<AppState>) -> Json<DashboardHealth> {
    let api_health = state.api.health().await.ok();
    Json(DashboardHealth {
        service: "eal-leptos-web",
        status: if api_health.is_some() {
            "degraded"
        } else {
            "unavailable"
        },
        environment: state.environment.as_str(),
        production_ready: false,
        tenant_context: "development_header",
        api_reachable: api_health.is_some(),
        api_production_ready: api_health.map(|health| health.production_ready),
    })
}

async fn load_dashboard(
    state: &AppState,
    query_text: String,
    preview_request: Option<SemanticSearchRequest>,
    local_error: Option<String>,
) -> DashboardData {
    let preview_future = async {
        match preview_request {
            Some(request) => Some(state.api.search(&request).await),
            None => None,
        }
    };
    let (health, sources, pages, matches, preview) = tokio::join!(
        state.api.health(),
        state.api.list_sources(),
        state.api.list_pages(),
        state.api.list_matches(),
        preview_future,
    );

    let mut errors = Vec::new();
    if let Some(error) = local_error {
        errors.push(error);
    }
    let health = capture("health", health, &mut errors);
    let sources = capture("sources", sources, &mut errors).unwrap_or_default();
    let pages = capture("pages", pages, &mut errors).unwrap_or_default();
    let matches = capture("matches", matches, &mut errors).unwrap_or_default();
    let preview = preview.and_then(|result| capture("preview", result, &mut errors));

    DashboardData {
        health,
        sources,
        pages,
        matches,
        preview,
        query_text,
        errors,
        environment: state.environment.as_str().into(),
        tenant_label: short_uuid(state.tenant_id),
    }
}

fn capture<T>(
    label: &str,
    result: Result<T, ApiClientError>,
    errors: &mut Vec<String>,
) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            errors.push(format!("{label}: {} · {}", error.code, error.message));
            None
        }
    }
}

fn render_page(data: DashboardData) -> Html<String> {
    use leptos::tachys::view::RenderHtml;

    let body = view! { <Dashboard data=data/> }.to_html();
    Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"color-scheme\" content=\"dark\"><title>Embedded Alerts · Interests</title><style>{STYLES}</style></head><body>{body}</body></html>"
    ))
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

fn compact_json(value: &serde_json::Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "unknown".into())
}

fn short_uuid(value: Uuid) -> String {
    value.to_string().chars().take(8).collect()
}

fn short_hash(value: &str) -> String {
    if value.len() <= 16 {
        value.to_owned()
    } else {
        format!("{}…{}", &value[..8], &value[value.len() - 8..])
    }
}

fn timestamp(value: chrono::DateTime<chrono::Utc>) -> String {
    value.format("%Y-%m-%d %H:%M UTC").to_string()
}

fn percent(value: f32) -> String {
    format!("{:.1}%", value.clamp(0.0, 1.0) * 100.0)
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

const STYLES: &str = r#"
:root{--bg:#f0efe8;--ink:#17211c;--muted:#6a716c;--line:#c8cbc2;--green:#0b6b47;--lime:#d8f08b;--red:#9d3027;font-family:Inter,ui-sans-serif,system-ui,sans-serif;color:var(--ink);background:var(--bg)}*{box-sizing:border-box}body{margin:0;background:var(--bg)}a{color:inherit;text-decoration-color:var(--green);text-underline-offset:.2em}code{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;overflow-wrap:anywhere}.shell{width:min(1260px,calc(100% - 2rem));margin:auto;padding:2rem 0 6rem}.topbar{display:flex;justify-content:space-between;gap:2rem;align-items:end;border-bottom:2px solid var(--ink);padding:3rem 0 1.5rem}.eyebrow,.section-number,.kicker{margin:0 0 .5rem;color:var(--green);font:700 .72rem/1.4 ui-monospace,monospace;letter-spacing:.14em}.topbar h1{max-width:850px;margin:0;font-size:clamp(3rem,8vw,7rem);line-height:.88;letter-spacing:-.065em;font-weight:650}.runtime{text-align:right;color:var(--muted);text-transform:uppercase;font-size:.72rem}.runtime span,.runtime code{display:block}.hero{display:grid;grid-template-columns:1.3fr .7fr;gap:4rem;padding:2rem 0}.hero>p{font-size:1.25rem;line-height:1.55}.lock-note{border-left:5px solid var(--red);padding:1rem 0 1rem 1rem}.lock-note span{display:block;color:var(--muted);margin-top:.4rem}.stats{display:grid;grid-template-columns:repeat(4,1fr);border:1px solid var(--line)}.stats article{min-height:130px;padding:1rem;border-right:1px solid var(--line);display:flex;flex-direction:column;justify-content:space-between}.stats article:last-child{border-right:0}.stats strong{font-size:2rem}.stats span,.stats small{color:var(--muted)}.health-card code{font-size:.65rem}.panel{border-top:1px solid var(--ink);padding:2.5rem 0}.panel-head,.card-head{display:flex;justify-content:space-between;gap:1rem;align-items:start}.panel-head{margin-bottom:1.25rem}.panel h2{margin:0;font-size:clamp(1.8rem,4vw,3.6rem);letter-spacing:-.045em}.pill{border:1px solid var(--green);padding:.25rem .55rem;color:var(--green);font:.68rem ui-monospace,monospace;text-transform:uppercase}.pill--locked{border-color:var(--red);color:var(--red)}.preview-panel{background:var(--lime);padding-inline:clamp(1rem,4vw,3rem)}.preview-form{display:grid;gap:1rem;max-width:900px}.preview-form label>span{display:block;margin-bottom:.35rem;font-size:.72rem;text-transform:uppercase;letter-spacing:.08em}.preview-form textarea,.preview-form input{width:100%;border:1px solid var(--ink);border-radius:0;background:#fffef7;padding:.8rem;font:inherit}.form-row{display:grid;grid-template-columns:repeat(2,1fr);gap:1rem}.preview-form button{justify-self:start;border:0;background:var(--ink);color:white;padding:.8rem 1.2rem;font-weight:700}.preview-results{margin-top:2rem;border-top:1px solid var(--ink);padding-top:1.5rem}.preview-summary{display:grid;gap:.25rem;margin-bottom:1rem}.preview-summary small{color:var(--muted)}.source-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:1rem}.source-card,.revision-card,.candidate-card,.result-card{border:1px solid var(--line);background:#faf9f3;padding:1rem}.source-card--disabled{opacity:.58}.source-card h3,.revision-card h3,.candidate-card h3,.result-card h3{margin:.15rem 0 .45rem}.source-card p{color:var(--muted)}dl{display:grid;gap:.4rem;margin:1rem 0}dl>div{display:grid;grid-template-columns:130px 1fr;gap:.75rem;border-top:1px solid var(--line);padding-top:.35rem}dt{color:var(--muted);font-size:.7rem;text-transform:uppercase}dd{margin:0}.revision-list,.candidate-list,.result-list{display:grid;gap:1rem}.revision-card small{display:block;color:var(--muted);margin-top:.35rem}.candidate-card,.result-card{display:grid;grid-template-columns:125px 1fr;gap:1rem}.candidate-score{border-right:1px solid var(--line)}.candidate-score strong,.candidate-score span{display:block}.candidate-score strong{font-size:1.6rem;color:var(--green)}.candidate-score span{color:var(--muted);font-size:.7rem;text-transform:uppercase}.score-grid{display:grid;grid-template-columns:repeat(5,1fr);gap:.5rem;margin:1rem 0}.score-grid>div{border-top:1px solid var(--line);padding-top:.35rem}.score-grid span,.score-grid strong{display:block}.score-grid span{color:var(--muted);font-size:.65rem;text-transform:uppercase}meter{width:100%;height:.35rem;accent-color:var(--green)}.evidence li{margin-bottom:.75rem}.evidence li span{display:block;color:var(--muted);font-size:.72rem}.evidence li p{margin:.25rem 0}.empty{border:1px dashed var(--line);padding:1rem;color:var(--muted)}.empty strong{color:var(--ink)}.notice{border-left:5px solid var(--red);background:#f8ded9;padding:.9rem 1rem;margin:1rem 0}.notice p{margin:.35rem 0 0}.quiet-link{font-size:.75rem;text-transform:uppercase}@media(max-width:900px){.hero,.source-grid{grid-template-columns:1fr}.stats{grid-template-columns:repeat(2,1fr)}.score-grid{grid-template-columns:repeat(2,1fr)}}@media(max-width:620px){.shell{width:calc(100% - 1rem)}.topbar,.panel-head,.card-head{flex-direction:column}.stats,.form-row{grid-template-columns:1fr}.stats article{border-right:0;border-bottom:1px solid var(--line)}.candidate-card,.result-card{grid-template-columns:1fr}.candidate-score{border-right:0;border-bottom:1px solid var(--line);padding-bottom:.5rem}}
"#;

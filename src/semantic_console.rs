use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceView {
    pub id: String,
    pub domain: String,
    pub status: String,
    pub include_subdomains: bool,
    pub respect_robots: bool,
    pub page_budget: u32,
    pub indexed_pages: u64,
    pub last_scan_label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuleView {
    pub id: String,
    pub name: String,
    pub revision: u32,
    pub query_text: String,
    pub threshold: f32,
    pub candidate_count: u64,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CandidateView {
    pub id: String,
    pub rule_name: String,
    pub page_title: String,
    pub canonical_url: String,
    pub source_domain: String,
    pub state: String,
    pub overall_score: f32,
    pub semantic_score: f32,
    pub lexical_score: f32,
    pub entity_score: f32,
    pub recency_score: f32,
    pub source_priority_score: f32,
    pub best_sentence: String,
    pub entities: Vec<String>,
    pub keywords: Vec<String>,
    pub model_label: String,
    pub discovered_label: String,
}

#[component]
pub fn SemanticConsole(
    tenant_name: String,
    sources: Vec<SourceView>,
    rules: Vec<RuleView>,
    candidates: Vec<CandidateView>,
    csrf_token: String,
) -> impl IntoView {
    let source_cards = sources
        .into_iter()
        .map(|source| {
            let subdomain_label = if source.include_subdomains {
                "Exact host + subdomains"
            } else {
                "Exact host only"
            };
            let robots_label = if source.respect_robots {
                "Robots enforced"
            } else {
                "Robots disabled"
            };
            let detail_url = format!("/sources/{}", source.id);
            let scan_url = format!("/ui/sources/{}/scan", source.id);
            let status_class = format!("status-pill status-{}", source.status);
            view! {
                <article class="source-card panel" data-source-id=source.id>
                    <div class="card-heading">
                        <div>
                            <p class="domain-label">{source.domain}</p>
                            <h3>{subdomain_label}</h3>
                        </div>
                        <span class=status_class>{source.status}</span>
                    </div>
                    <dl class="metric-list">
                        <div><dt>"Indexed pages"</dt><dd>{source.indexed_pages}</dd></div>
                        <div><dt>"Page budget"</dt><dd>{source.page_budget}</dd></div>
                        <div><dt>"Robots"</dt><dd>{robots_label}</dd></div>
                        <div><dt>"Last scan"</dt><dd>{source.last_scan_label}</dd></div>
                    </dl>
                    <div class="card-actions">
                        <form action=scan_url method="post">
                            <input type="hidden" name="csrf_token" value=csrf_token.clone()/>
                            <button type="submit">"Scan now"</button>
                        </form>
                        <a href=detail_url>"Inspect pages"</a>
                    </div>
                </article>
            }
        })
        .collect_view();

    let rule_cards = rules
        .into_iter()
        .map(|rule| {
            let evaluate_url = format!("/ui/alert-rules/{}/evaluate", rule.id);
            let detail_url = format!("/alert-rules/{}", rule.id);
            let status = if rule.enabled { "enabled" } else { "paused" };
            let status_class = format!("status-pill status-{status}");
            let threshold = format!("{:.0}%", clamp_score(rule.threshold) * 100.0);
            view! {
                <article class="rule-card panel" data-rule-id=rule.id>
                    <div class="card-heading">
                        <div>
                            <p class="revision-label">{format!("Revision {}", rule.revision)}</p>
                            <h3>{rule.name}</h3>
                        </div>
                        <span class=status_class>{status}</span>
                    </div>
                    <blockquote>{rule.query_text}</blockquote>
                    <dl class="metric-list">
                        <div><dt>"Threshold"</dt><dd>{threshold}</dd></div>
                        <div><dt>"Candidates"</dt><dd>{rule.candidate_count}</dd></div>
                    </dl>
                    <div class="card-actions">
                        <form action=evaluate_url method="post">
                            <input type="hidden" name="csrf_token" value=csrf_token.clone()/>
                            <button type="submit">"Evaluate new pages"</button>
                        </form>
                        <a href=detail_url>"Revision history"</a>
                    </div>
                </article>
            }
        })
        .collect_view();

    let candidate_cards = candidates
        .into_iter()
        .map(|candidate| {
            let approve_url = format!("/ui/matches/{}/approve", candidate.id);
            let suppress_url = format!("/ui/matches/{}/suppress", candidate.id);
            let dismiss_url = format!("/ui/matches/{}/dismiss", candidate.id);
            let detail_url = format!("/matches/{}", candidate.id);
            let status_class = format!("status-pill status-{}", candidate.state);
            let overall_score = clamp_score(candidate.overall_score);
            let score_percent = format!("{:.0}", overall_score * 100.0);
            let entity_tags = candidate
                .entities
                .into_iter()
                .map(|entity| view! { <li class="entity-tag">{entity}</li> })
                .collect_view();
            let keyword_tags = candidate
                .keywords
                .into_iter()
                .map(|keyword| view! { <li class="keyword-tag">{keyword}</li> })
                .collect_view();
            view! {
                <article class="match-card panel" data-match-id=candidate.id>
                    <div class="match-heading">
                        <div>
                            <p class="match-context">{format!("{} · {}", candidate.rule_name, candidate.source_domain)}</p>
                            <h3>
                                <a href=candidate.canonical_url target="_blank" rel="noopener noreferrer">
                                    {candidate.page_title}
                                </a>
                            </h3>
                        </div>
                        <div class="score-badge" aria-label=format!("Overall match score {} percent", score_percent)>
                            <strong>{score_percent}</strong><span>"/ 100"</span>
                        </div>
                    </div>
                    <meter min="0" max="1" value=overall_score aria-label="Overall semantic match"></meter>
                    <blockquote class="sentence-evidence">
                        <span>"Best complete-sentence evidence"</span>
                        {candidate.best_sentence}
                    </blockquote>
                    <div class="evidence-grid">
                        <section aria-label="Score components">
                            <h4>"Why it matched"</h4>
                            <ScoreRow label="Semantic" score=candidate.semantic_score/>
                            <ScoreRow label="Lexical" score=candidate.lexical_score/>
                            <ScoreRow label="Entity" score=candidate.entity_score/>
                            <ScoreRow label="Recency" score=candidate.recency_score/>
                            <ScoreRow label="Source priority" score=candidate.source_priority_score/>
                        </section>
                        <section aria-label="Matched concepts">
                            <h4>"Concept evidence"</h4>
                            <div class="tag-group"><span>"Entities"</span><ul>{entity_tags}</ul></div>
                            <div class="tag-group"><span>"Keywords"</span><ul>{keyword_tags}</ul></div>
                        </section>
                    </div>
                    <footer class="match-footer">
                        <div>
                            <span class=status_class>{candidate.state}</span>
                            <span>{candidate.discovered_label}</span>
                            <span>{candidate.model_label}</span>
                        </div>
                        <div class="card-actions">
                            <form action=approve_url method="post">
                                <input type="hidden" name="csrf_token" value=csrf_token.clone()/>
                                <input type="hidden" name="expected_state" value="candidate"/>
                                <button class="primary-action" type="submit">"Approve"</button>
                            </form>
                            <form action=suppress_url method="post">
                                <input type="hidden" name="csrf_token" value=csrf_token.clone()/>
                                <input type="hidden" name="expected_state" value="candidate"/>
                                <button type="submit">"Suppress"</button>
                            </form>
                            <form action=dismiss_url method="post">
                                <input type="hidden" name="csrf_token" value=csrf_token.clone()/>
                                <input type="hidden" name="expected_state" value="candidate"/>
                                <button type="submit">"Dismiss"</button>
                            </form>
                            <a href=detail_url>"Full evidence"</a>
                        </div>
                    </footer>
                </article>
            }
        })
        .collect_view();

    view! {
        <main class="semantic-console" data-component="semantic-console">
            <header class="console-header">
                <div>
                    <p class="eyebrow">{format!("Embedded Alerts / {tenant_name}")}</p>
                    <h1>"Semantic monitoring console"</h1>
                    <p class="lede">
                        "Index approved public domains, describe what matters in natural language, and review explainable matches before delivery."
                    </p>
                </div>
                <nav class="section-nav" aria-label="Semantic console sections">
                    <a href="#sources">"Sources"</a>
                    <a href="#rules">"Alert rules"</a>
                    <a href="#matches">"Match candidates"</a>
                </nav>
            </header>

            <section id="sources" class="console-section">
                <div class="section-heading">
                    <div><p class="section-kicker">"Discovery boundary"</p><h2>"Approved domains"</h2></div>
                    <p>"Only configured public hosts are eligible. External indexes may suggest URLs, but every page is fetched and checked against this policy."</p>
                </div>
                <form class="source-form panel" action="/ui/sources" method="post">
                    <input type="hidden" name="csrf_token" value=csrf_token.clone()/>
                    <div class="field-grid">
                        <label><span>"Public domain"</span><input name="domain" type="text" maxlength="253" required/></label>
                        <label><span>"Seed URLs"</span><textarea name="seed_urls" rows="3" maxlength="8000"></textarea></label>
                        <label><span>"Pages per scan"</span><input name="page_budget" type="number" min="1" max="5000" value="250" required/></label>
                        <label><span>"Source priority"</span><input name="priority" type="number" min="0" max="100" value="50" required/></label>
                    </div>
                    <fieldset class="choice-row">
                        <legend>"Policy"</legend>
                        <label><input type="checkbox" name="include_subdomains" value="true"/>"Include subdomains"</label>
                        <label><input type="checkbox" name="respect_robots" value="true" checked/>"Enforce robots.txt"</label>
                        <label><input type="checkbox" name="discover_sitemaps" value="true" checked/>"Discover sitemaps"</label>
                        <label><input type="checkbox" name="discover_links" value="true" checked/>"Follow bounded same-domain links"</label>
                    </fieldset>
                    <button class="primary-action" type="submit">"Register source"</button>
                </form>
                <div class="card-grid">{source_cards}</div>
            </section>

            <section id="rules" class="console-section">
                <div class="section-heading">
                    <div><p class="section-kicker">"Semantic intent"</p><h2>"Natural-language alert rules"</h2></div>
                    <p>"The complete sentence remains the strongest representation. Keywords and proper nouns are companion evidence, not replacements."</p>
                </div>
                <form class="rule-form panel" action="/ui/alert-rules" method="post">
                    <input type="hidden" name="csrf_token" value=csrf_token.clone()/>
                    <div class="field-grid">
                        <label><span>"Rule name"</span><input name="name" type="text" maxlength="120" required/></label>
                        <label class="wide-field"><span>"What should Embedded Alerts find?"</span><textarea name="query_text" rows="4" minlength="3" maxlength="700" required></textarea></label>
                        <label><span>"Candidate threshold"</span><input name="threshold" type="number" min="0" max="1" step="0.01" value="0.72" required/></label>
                    </div>
                    <button class="primary-action" type="submit">"Create immutable revision"</button>
                    <button formaction="/ui/query-preview" type="submit">"Preview semantic views"</button>
                </form>
                <div class="card-grid">{rule_cards}</div>
            </section>

            <section id="matches" class="console-section">
                <div class="section-heading">
                    <div><p class="section-kicker">"Explainable ranking"</p><h2>"Match candidates"</h2></div>
                    <p>"Review semantic, lexical, entity, recency, and source-priority evidence before a candidate enters delivery."</p>
                </div>
                <div class="match-list" aria-live="polite">{candidate_cards}</div>
            </section>
        </main>
    }
}

#[component]
fn ScoreRow(label: &'static str, score: f32) -> impl IntoView {
    let score = clamp_score(score);
    let percent = format!("{:.0}%", score * 100.0);
    view! {
        <div class="score-row">
            <span>{label}</span>
            <meter min="0" max="1" value=score></meter>
            <strong>{percent}</strong>
        </div>
    }
}

fn clamp_score(score: f32) -> f32 {
    if score.is_finite() {
        score.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub fn browser_contract_field_names() -> &'static [&'static str] {
    &[
        "domain",
        "seed_urls",
        "page_budget",
        "priority",
        "query_text",
        "threshold",
        "expected_state",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_contract_contains_no_embedding_values() {
        let fields = browser_contract_field_names();
        assert!(!fields.contains(&"vector"));
        assert!(!fields.contains(&"embedding"));
        assert!(!fields.contains(&"dimensions"));
    }

    #[test]
    fn non_finite_scores_fail_closed() {
        assert_eq!(clamp_score(f32::NAN), 0.0);
        assert_eq!(clamp_score(f32::INFINITY), 0.0);
        assert_eq!(clamp_score(-1.0), 0.0);
        assert_eq!(clamp_score(2.0), 1.0);
    }
}

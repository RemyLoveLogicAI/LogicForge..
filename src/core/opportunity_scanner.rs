//! Opportunity Scanner - Discovers opportunities from various sources

use crate::db::Database;
use crate::models::{
    Opportunity, OpportunityBuilder, OpportunitySource, RawOpportunityData,
};
use crate::utils::config::GenesisConfig;
use crate::utils::error::{GenesisError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Scanner trait for implementing different source scanners
#[async_trait]
pub trait Scanner: Send + Sync {
    /// Get the source type
    fn source(&self) -> OpportunitySource;

    /// Scan for opportunities
    async fn scan(&self, limit: Option<usize>) -> Result<Vec<Opportunity>>;

    /// Check if the scanner is configured and ready
    fn is_ready(&self) -> bool;
}

/// Configuration for the opportunity scanner
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    pub enabled_sources: Vec<String>,
    pub min_score: f64,
    pub keywords: Vec<String>,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            enabled_sources: vec![
                "github".to_string(),
                "reddit".to_string(),
                "hackernews".to_string(),
            ],
            min_score: 50.0,
            keywords: vec![
                "looking for".to_string(),
                "wish there was".to_string(),
                "need a tool".to_string(),
                "automate".to_string(),
                "saas".to_string(),
            ],
        }
    }
}

/// Main opportunity scanner that orchestrates multiple source scanners
#[allow(dead_code)]
pub struct OpportunityScanner {
    config: ScannerConfig,
    http_client: Client,
    db: Arc<Database>,
    scanners: Vec<Box<dyn Scanner>>,
    seen_ids: HashSet<String>,
}

impl OpportunityScanner {
    /// Create a new opportunity scanner
    pub fn new(config: ScannerConfig, db: Arc<Database>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Genesis-Engine/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client: http_client.clone(),
            db,
            scanners: Vec::new(),
            seen_ids: HashSet::new(),
        }
    }

    /// Create from genesis config
    pub fn from_config(genesis_config: &GenesisConfig, db: Arc<Database>) -> Self {
        let config = ScannerConfig {
            enabled_sources: genesis_config.scanning.enabled_sources.clone(),
            min_score: genesis_config.scanning.min_opportunity_score,
            keywords: vec![],
        };

        let mut scanner = Self::new(config, db);

        // Add scanners based on configuration
        for source in &genesis_config.scanning.enabled_sources {
            match source.to_lowercase().as_str() {
                "github" => {
                    if let Some(token) = &genesis_config.github.token {
                        scanner.add_scanner(Box::new(GitHubScanner::new(
                            scanner.http_client.clone(),
                            token.clone(),
                        )));
                    }
                }
                "hackernews" | "hn" => {
                    scanner.add_scanner(Box::new(HackerNewsScanner::new(
                        scanner.http_client.clone(),
                    )));
                }
                "reddit" => {
                    scanner.add_scanner(Box::new(RedditScanner::new(
                        scanner.http_client.clone(),
                    )));
                }
                _ => {
                    warn!("Unknown scanner source: {}", source);
                }
            }
        }

        scanner
    }

    /// Add a scanner
    pub fn add_scanner(&mut self, scanner: Box<dyn Scanner>) {
        self.scanners.push(scanner);
    }

    /// Scan all configured sources
    pub async fn scan_all_sources(&mut self, limit: Option<usize>) -> Result<Vec<Opportunity>> {
        let mut all_opportunities = Vec::new();

        for scanner in &self.scanners {
            if !scanner.is_ready() {
                warn!("Scanner {:?} is not ready, skipping", scanner.source());
                continue;
            }

            info!("Scanning {:?}...", scanner.source());

            match scanner.scan(limit).await {
                Ok(opportunities) => {
                    info!(
                        "Found {} opportunities from {:?}",
                        opportunities.len(),
                        scanner.source()
                    );

                    for opp in opportunities {
                        // Deduplicate
                        if !self.seen_ids.contains(&opp.id) {
                            self.seen_ids.insert(opp.id.clone());
                            all_opportunities.push(opp);
                        }
                    }
                }
                Err(e) => {
                    warn!("Error scanning {:?}: {}", scanner.source(), e);
                }
            }
        }

        // Sort by score (highest first)
        all_opportunities.sort_by(|a, b| {
            b.score
                .unwrap_or(0.0)
                .partial_cmp(&a.score.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply limit
        if let Some(l) = limit {
            all_opportunities.truncate(l);
        }

        Ok(all_opportunities)
    }

    /// Score an opportunity using heuristics
    pub fn score_opportunity(&self, opp: &mut Opportunity) {
        let mut score = 50.0; // Base score

        // Check for keywords in title and description
        let text = format!("{} {}", opp.title, opp.description).to_lowercase();

        let positive_keywords = [
            "automation",
            "saas",
            "api",
            "tool",
            "platform",
            "dashboard",
            "ai",
            "machine learning",
            "analytics",
            "integration",
        ];

        let negative_keywords = [
            "homework",
            "assignment",
            "free",
            "illegal",
            "crack",
            "pirate",
        ];

        for keyword in positive_keywords {
            if text.contains(keyword) {
                score += 5.0;
            }
        }

        for keyword in negative_keywords {
            if text.contains(keyword) {
                score -= 10.0;
            }
        }

        // Boost for engagement signals
        if let Some(upvotes) = opp.raw_data.upvotes {
            score += (upvotes as f64).log10() * 5.0;
        }

        if let Some(comments) = opp.raw_data.comments_count {
            score += (comments as f64).log10() * 3.0;
        }

        // Pain points boost
        score += opp.pain_points.len() as f64 * 5.0;

        // Clamp score
        opp.set_score(score.clamp(0.0, 100.0));
    }

    /// Save an opportunity to the database
    pub async fn save_opportunity(&self, opp: &Opportunity) -> Result<()> {
        self.db.save_opportunity(opp)?;
        Ok(())
    }

    /// Save multiple opportunities
    pub async fn save_opportunities(&self, opportunities: &[Opportunity]) -> Result<()> {
        for opp in opportunities {
            self.save_opportunity(opp).await?;
        }
        Ok(())
    }

    /// Get the HTTP client
    pub fn http_client(&self) -> &Client {
        &self.http_client
    }
}

// ==================== GitHub Scanner ====================

/// GitHub trending and issues scanner
pub struct GitHubScanner {
    http_client: Client,
    token: String,
}

impl GitHubScanner {
    pub fn new(http_client: Client, token: String) -> Self {
        Self { http_client, token }
    }

    async fn fetch_trending_repos(&self, limit: usize) -> Result<Vec<GitHubRepo>> {
        // Search for trending repos by stars created recently
        let url = format!(
            "https://api.github.com/search/repositories?q=created:>2024-01-01&sort=stars&order=desc&per_page={}",
            limit.min(100)
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(GenesisError::api(
                format!("GitHub API error: {}", response.status()),
                response.status().as_u16(),
            ));
        }

        let data: GitHubSearchResponse = response.json().await?;
        Ok(data.items)
    }

    async fn fetch_issues_with_labels(&self, labels: &[&str], limit: usize) -> Result<Vec<GitHubIssue>> {
        let labels_query = labels.join(",");
        let url = format!(
            "https://api.github.com/search/issues?q=label:{}&sort=created&order=desc&per_page={}",
            labels_query,
            limit.min(100)
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(Vec::new()); // Gracefully handle API errors
        }

        let data: GitHubIssuesResponse = response.json().await?;
        Ok(data.items)
    }
}

#[async_trait]
impl Scanner for GitHubScanner {
    fn source(&self) -> OpportunitySource {
        OpportunitySource::GitHub
    }

    async fn scan(&self, limit: Option<usize>) -> Result<Vec<Opportunity>> {
        let limit = limit.unwrap_or(20);
        let mut opportunities = Vec::new();

        // Scan trending repos
        if let Ok(repos) = self.fetch_trending_repos(limit / 2).await {
            for repo in repos {
                let opp = OpportunityBuilder::new()
                    .title(format!("Trending: {}", repo.name))
                    .description(repo.description.unwrap_or_default())
                    .source(OpportunitySource::GitHub)
                    .url(repo.html_url)
                    .raw_data(RawOpportunityData {
                        upvotes: Some(repo.stargazers_count),
                        tags: repo.topics.unwrap_or_default(),
                        author: Some(repo.owner.login),
                        ..Default::default()
                    })
                    .build()?;

                opportunities.push(opp);
            }
        }

        // Scan issues with "help wanted" or "good first issue"
        if let Ok(issues) = self.fetch_issues_with_labels(&["help-wanted", "good-first-issue"], limit / 2).await {
            for issue in issues {
                let opp = OpportunityBuilder::new()
                    .title(format!("Issue: {}", issue.title))
                    .description(issue.body.unwrap_or_default())
                    .source(OpportunitySource::GitHub)
                    .url(issue.html_url)
                    .raw_data(RawOpportunityData {
                        comments_count: Some(issue.comments),
                        author: Some(issue.user.login),
                        ..Default::default()
                    })
                    .build()?;

                opportunities.push(opp);
            }
        }

        Ok(opportunities)
    }

    fn is_ready(&self) -> bool {
        !self.token.is_empty()
    }
}

#[derive(Debug, Deserialize)]
struct GitHubSearchResponse {
    items: Vec<GitHubRepo>,
}

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    name: String,
    description: Option<String>,
    html_url: String,
    stargazers_count: i64,
    topics: Option<Vec<String>>,
    owner: GitHubOwner,
}

#[derive(Debug, Deserialize)]
struct GitHubOwner {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GitHubIssuesResponse {
    items: Vec<GitHubIssue>,
}

#[derive(Debug, Deserialize)]
struct GitHubIssue {
    title: String,
    body: Option<String>,
    html_url: String,
    comments: i64,
    user: GitHubOwner,
}

// ==================== Hacker News Scanner ====================

/// Hacker News scanner
pub struct HackerNewsScanner {
    http_client: Client,
}

impl HackerNewsScanner {
    pub fn new(http_client: Client) -> Self {
        Self { http_client }
    }

    async fn fetch_top_stories(&self, limit: usize) -> Result<Vec<HNStory>> {
        // Get top story IDs
        let ids_response = self
            .http_client
            .get("https://hacker-news.firebaseio.com/v0/topstories.json")
            .send()
            .await?;

        let ids: Vec<u64> = ids_response.json().await?;

        // Fetch details for each story (limited)
        let mut stories = Vec::new();
        for id in ids.into_iter().take(limit) {
            if let Ok(story) = self.fetch_story(id).await {
                stories.push(story);
            }
        }

        Ok(stories)
    }

    async fn fetch_story(&self, id: u64) -> Result<HNStory> {
        let url = format!(
            "https://hacker-news.firebaseio.com/v0/item/{}.json",
            id
        );

        let response = self.http_client.get(&url).send().await?;
        let story: HNStory = response.json().await?;
        Ok(story)
    }

    async fn fetch_ask_hn(&self, limit: usize) -> Result<Vec<HNStory>> {
        let ids_response = self
            .http_client
            .get("https://hacker-news.firebaseio.com/v0/askstories.json")
            .send()
            .await?;

        let ids: Vec<u64> = ids_response.json().await?;

        let mut stories = Vec::new();
        for id in ids.into_iter().take(limit) {
            if let Ok(story) = self.fetch_story(id).await {
                stories.push(story);
            }
        }

        Ok(stories)
    }
}

#[async_trait]
impl Scanner for HackerNewsScanner {
    fn source(&self) -> OpportunitySource {
        OpportunitySource::HackerNews
    }

    async fn scan(&self, limit: Option<usize>) -> Result<Vec<Opportunity>> {
        let limit = limit.unwrap_or(20);
        let mut opportunities = Vec::new();

        // Scan Ask HN posts (often contain problem statements)
        if let Ok(stories) = self.fetch_ask_hn(limit / 2).await {
            for story in stories {
                if let Some(title) = story.title {
                    // Filter for potential opportunities
                    let title_lower = title.to_lowercase();
                    if title_lower.contains("ask hn")
                        || title_lower.contains("looking for")
                        || title_lower.contains("need")
                        || title_lower.contains("recommend")
                    {
                        let opp = OpportunityBuilder::new()
                            .title(title)
                            .description(story.text.unwrap_or_default())
                            .source(OpportunitySource::HackerNews)
                            .url(format!("https://news.ycombinator.com/item?id={}", story.id))
                            .raw_data(RawOpportunityData {
                                upvotes: story.score.map(|s| s as i64),
                                comments_count: story.descendants.map(|d| d as i64),
                                author: story.by,
                                ..Default::default()
                            })
                            .build()?;

                        opportunities.push(opp);
                    }
                }
            }
        }

        // Scan Show HN posts (for inspiration and competition analysis)
        if let Ok(stories) = self.fetch_top_stories(limit / 2).await {
            for story in stories {
                if let Some(title) = story.title {
                    if title.to_lowercase().contains("show hn") {
                        let opp = OpportunityBuilder::new()
                            .title(title)
                            .description(story.text.unwrap_or_default())
                            .source(OpportunitySource::HackerNews)
                            .url(story.url.unwrap_or_else(|| {
                                format!("https://news.ycombinator.com/item?id={}", story.id)
                            }))
                            .raw_data(RawOpportunityData {
                                upvotes: story.score.map(|s| s as i64),
                                comments_count: story.descendants.map(|d| d as i64),
                                author: story.by,
                                ..Default::default()
                            })
                            .build()?;

                        opportunities.push(opp);
                    }
                }
            }
        }

        Ok(opportunities)
    }

    fn is_ready(&self) -> bool {
        true // No auth required for HN API
    }
}

#[derive(Debug, Deserialize)]
struct HNStory {
    id: u64,
    title: Option<String>,
    text: Option<String>,
    url: Option<String>,
    score: Option<u32>,
    by: Option<String>,
    descendants: Option<u32>,
}

// ==================== Reddit Scanner ====================

/// Reddit scanner (using public JSON endpoints)
pub struct RedditScanner {
    http_client: Client,
}

impl RedditScanner {
    pub fn new(http_client: Client) -> Self {
        Self { http_client }
    }

    async fn fetch_subreddit(&self, subreddit: &str, limit: usize) -> Result<Vec<RedditPost>> {
        let url = format!(
            "https://www.reddit.com/r/{}/hot.json?limit={}",
            subreddit,
            limit.min(100)
        );

        let response = self
            .http_client
            .get(&url)
            .header("User-Agent", "Genesis-Engine/0.1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let data: RedditResponse = response.json().await?;
        Ok(data.data.children.into_iter().map(|c| c.data).collect())
    }
}

#[async_trait]
impl Scanner for RedditScanner {
    fn source(&self) -> OpportunitySource {
        OpportunitySource::Reddit
    }

    async fn scan(&self, limit: Option<usize>) -> Result<Vec<Opportunity>> {
        let limit = limit.unwrap_or(20);
        let mut opportunities = Vec::new();

        // Scan relevant subreddits
        let subreddits = [
            "SaaS",
            "startups",
            "Entrepreneur",
            "microsaas",
            "IndieHackers",
            "SideProject",
        ];

        let per_sub = (limit / subreddits.len()).max(5);

        for subreddit in subreddits {
            debug!("Scanning r/{}", subreddit);

            if let Ok(posts) = self.fetch_subreddit(subreddit, per_sub).await {
                for post in posts {
                    // Filter for potential opportunities
                    let title_lower = post.title.to_lowercase();
                    if title_lower.contains("looking for")
                        || title_lower.contains("need")
                        || title_lower.contains("built")
                        || title_lower.contains("launched")
                        || title_lower.contains("idea")
                        || title_lower.contains("feedback")
                    {
                        let opp = OpportunityBuilder::new()
                            .title(post.title)
                            .description(post.selftext.unwrap_or_default())
                            .source(OpportunitySource::Reddit)
                            .url(format!("https://reddit.com{}", post.permalink))
                            .raw_data(RawOpportunityData {
                                upvotes: Some(post.score as i64),
                                comments_count: Some(post.num_comments as i64),
                                author: Some(post.author),
                                subreddit: Some(subreddit.to_string()),
                                ..Default::default()
                            })
                            .build()?;

                        opportunities.push(opp);
                    }
                }
            }

            // Rate limiting
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        Ok(opportunities)
    }

    fn is_ready(&self) -> bool {
        true // No auth required for public Reddit JSON
    }
}

#[derive(Debug, Deserialize)]
struct RedditResponse {
    data: RedditData,
}

#[derive(Debug, Deserialize)]
struct RedditData {
    children: Vec<RedditChild>,
}

#[derive(Debug, Deserialize)]
struct RedditChild {
    data: RedditPost,
}

#[derive(Debug, Deserialize)]
struct RedditPost {
    title: String,
    selftext: Option<String>,
    permalink: String,
    score: i32,
    num_comments: i32,
    author: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_config_default() {
        let config = ScannerConfig::default();
        assert!(!config.enabled_sources.is_empty());
        assert!(config.min_score > 0.0);
    }

    #[test]
    fn test_opportunity_scoring() {
        let db = Arc::new(Database::in_memory().unwrap());
        let scanner = OpportunityScanner::new(ScannerConfig::default(), db);

        let mut opp = Opportunity::new(
            "AI-powered SaaS dashboard for analytics",
            "A tool that helps automate business analytics",
            OpportunitySource::Manual,
        );

        scanner.score_opportunity(&mut opp);
        assert!(opp.score.is_some());
        assert!(opp.score.unwrap() > 50.0); // Should get boost from keywords
    }
}

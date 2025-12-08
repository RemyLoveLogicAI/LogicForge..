//! Opportunity model and related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of an opportunity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum OpportunityStatus {
    #[default]
    Pending,
    Evaluating,
    Approved,
    Rejected,
    Launched,
    Deferred,
}


impl std::fmt::Display for OpportunityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Evaluating => write!(f, "evaluating"),
            Self::Approved => write!(f, "approved"),
            Self::Rejected => write!(f, "rejected"),
            Self::Launched => write!(f, "launched"),
            Self::Deferred => write!(f, "deferred"),
        }
    }
}

impl std::str::FromStr for OpportunityStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "evaluating" => Ok(Self::Evaluating),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "launched" => Ok(Self::Launched),
            "deferred" => Ok(Self::Deferred),
            _ => Err(format!("Invalid opportunity status: {}", s)),
        }
    }
}

/// Source of the opportunity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OpportunitySource {
    GitHub,
    Reddit,
    ProductHunt,
    HackerNews,
    Twitter,
    Manual,
}

impl std::fmt::Display for OpportunitySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitHub => write!(f, "github"),
            Self::Reddit => write!(f, "reddit"),
            Self::ProductHunt => write!(f, "producthunt"),
            Self::HackerNews => write!(f, "hackernews"),
            Self::Twitter => write!(f, "twitter"),
            Self::Manual => write!(f, "manual"),
        }
    }
}

impl std::str::FromStr for OpportunitySource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "github" => Ok(Self::GitHub),
            "reddit" => Ok(Self::Reddit),
            "producthunt" | "product_hunt" => Ok(Self::ProductHunt),
            "hackernews" | "hacker_news" | "hn" => Ok(Self::HackerNews),
            "twitter" | "x" => Ok(Self::Twitter),
            "manual" => Ok(Self::Manual),
            _ => Ok(Self::Manual), // Default to Manual for unknown sources
        }
    }
}

/// Detected pain points from opportunity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PainPoint {
    pub description: String,
    pub severity: f64,
    pub frequency: String,
}

/// Market analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketAnalysis {
    pub size_estimate: String,
    pub growth_trend: String,
    pub competition_level: String,
    pub entry_barriers: Vec<String>,
}

/// Technical feasibility analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalAnalysis {
    pub complexity_score: f64,
    pub estimated_hours: u32,
    pub required_skills: Vec<String>,
    pub suggested_stack: Vec<String>,
}

/// Monetization analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonetizationAnalysis {
    pub clarity_score: f64,
    pub suggested_models: Vec<String>,
    pub price_range: Option<String>,
    pub potential_mrr: Option<f64>,
}

/// Raw data from the source
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawOpportunityData {
    pub url: Option<String>,
    pub author: Option<String>,
    pub upvotes: Option<i64>,
    pub comments_count: Option<i64>,
    pub created_at: Option<String>,
    pub subreddit: Option<String>,
    pub tags: Vec<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// An opportunity discovered by the scanner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opportunity {
    pub id: String,
    pub title: String,
    pub description: String,
    pub source: OpportunitySource,
    pub raw_data: RawOpportunityData,
    pub pain_points: Vec<PainPoint>,
    pub market_analysis: Option<MarketAnalysis>,
    pub technical_analysis: Option<TechnicalAnalysis>,
    pub monetization_analysis: Option<MonetizationAnalysis>,
    pub score: Option<f64>,
    pub status: OpportunityStatus,
    pub rejection_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Opportunity {
    /// Create a new opportunity
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        source: OpportunitySource,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            description: description.into(),
            source,
            raw_data: RawOpportunityData::default(),
            pain_points: Vec::new(),
            market_analysis: None,
            technical_analysis: None,
            monetization_analysis: None,
            score: None,
            status: OpportunityStatus::Pending,
            rejection_reason: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set raw data
    pub fn with_raw_data(mut self, data: RawOpportunityData) -> Self {
        self.raw_data = data;
        self
    }

    /// Add a pain point
    pub fn add_pain_point(&mut self, pain_point: PainPoint) {
        self.pain_points.push(pain_point);
    }

    /// Set the score
    pub fn set_score(&mut self, score: f64) {
        self.score = Some(score.clamp(0.0, 100.0));
        self.updated_at = Utc::now();
    }

    /// Update status
    pub fn set_status(&mut self, status: OpportunityStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Reject the opportunity
    pub fn reject(&mut self, reason: impl Into<String>) {
        self.status = OpportunityStatus::Rejected;
        self.rejection_reason = Some(reason.into());
        self.updated_at = Utc::now();
    }

    /// Check if opportunity is actionable
    pub fn is_actionable(&self) -> bool {
        matches!(
            self.status,
            OpportunityStatus::Pending | OpportunityStatus::Approved
        )
    }

    /// Get overall viability score
    pub fn viability_score(&self) -> f64 {
        self.score.unwrap_or(0.0)
    }
}

/// Ranked opportunity with decision data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedOpportunity {
    pub opportunity: Opportunity,
    pub rank: u32,
    pub reasoning: String,
    pub confidence: f64,
}

/// Builder for creating opportunities from scanner results
#[derive(Debug, Default)]
pub struct OpportunityBuilder {
    title: Option<String>,
    description: Option<String>,
    source: Option<OpportunitySource>,
    raw_data: RawOpportunityData,
    pain_points: Vec<PainPoint>,
}

impl OpportunityBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn source(mut self, source: OpportunitySource) -> Self {
        self.source = Some(source);
        self
    }

    pub fn raw_data(mut self, data: RawOpportunityData) -> Self {
        self.raw_data = data;
        self
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.raw_data.url = Some(url.into());
        self
    }

    pub fn add_pain_point(mut self, pain_point: PainPoint) -> Self {
        self.pain_points.push(pain_point);
        self
    }

    pub fn build(self) -> Result<Opportunity, String> {
        let title = self.title.ok_or("Title is required")?;
        let description = self.description.ok_or("Description is required")?;
        let source = self.source.ok_or("Source is required")?;

        let mut opp = Opportunity::new(title, description, source).with_raw_data(self.raw_data);
        opp.pain_points = self.pain_points;

        Ok(opp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_opportunity() {
        let opp = Opportunity::new(
            "Test Opportunity",
            "A test description",
            OpportunitySource::GitHub,
        );

        assert!(!opp.id.is_empty());
        assert_eq!(opp.title, "Test Opportunity");
        assert_eq!(opp.status, OpportunityStatus::Pending);
        assert!(opp.is_actionable());
    }

    #[test]
    fn test_opportunity_builder() {
        let opp = OpportunityBuilder::new()
            .title("Built Opportunity")
            .description("Built description")
            .source(OpportunitySource::Reddit)
            .url("https://reddit.com/r/test")
            .build()
            .unwrap();

        assert_eq!(opp.title, "Built Opportunity");
        assert_eq!(opp.source, OpportunitySource::Reddit);
        assert_eq!(opp.raw_data.url.unwrap(), "https://reddit.com/r/test");
    }

    #[test]
    fn test_score_clamping() {
        let mut opp = Opportunity::new("Test", "Test", OpportunitySource::Manual);
        opp.set_score(150.0);
        assert_eq!(opp.score, Some(100.0));

        opp.set_score(-10.0);
        assert_eq!(opp.score, Some(0.0));
    }

    #[test]
    fn test_status_parsing() {
        assert_eq!(
            "pending".parse::<OpportunityStatus>().unwrap(),
            OpportunityStatus::Pending
        );
        assert_eq!(
            "APPROVED".parse::<OpportunityStatus>().unwrap(),
            OpportunityStatus::Approved
        );
    }
}

//! Market Predictor - Analyzes market trends and makes predictions

use crate::models::{Opportunity, OpportunitySource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Market prediction confidence level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    Low,
    Medium,
    High,
}

impl ConfidenceLevel {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.8 {
            Self::High
        } else if score >= 0.5 {
            Self::Medium
        } else {
            Self::Low
        }
    }
}

/// Market trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketTrend {
    Rising,
    Stable,
    Declining,
    Volatile,
}

impl std::fmt::Display for MarketTrend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rising => write!(f, "📈 Rising"),
            Self::Stable => write!(f, "➡️ Stable"),
            Self::Declining => write!(f, "📉 Declining"),
            Self::Volatile => write!(f, "🔄 Volatile"),
        }
    }
}

/// Market prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPrediction {
    pub category: String,
    pub trend: MarketTrend,
    pub confidence: ConfidenceLevel,
    pub growth_potential: f64,
    pub competition_intensity: f64,
    pub entry_difficulty: f64,
    pub key_factors: Vec<String>,
    pub recommended_actions: Vec<String>,
}

/// Market segment analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSegment {
    pub name: String,
    pub size_estimate: String,
    pub growth_rate: f64,
    pub key_players: Vec<String>,
    pub entry_barriers: Vec<String>,
    pub opportunities: Vec<String>,
}

/// Configuration for market predictor
#[derive(Debug, Clone)]
pub struct PredictorConfig {
    /// Minimum data points for prediction
    pub min_data_points: usize,
    /// Historical days to consider
    pub lookback_days: u32,
    /// Weight for recent data vs historical
    pub recency_weight: f64,
}

impl Default for PredictorConfig {
    fn default() -> Self {
        Self {
            min_data_points: 5,
            lookback_days: 30,
            recency_weight: 0.7,
        }
    }
}

/// Market predictor for trend analysis
pub struct MarketPredictor {
    config: PredictorConfig,
    keyword_trends: HashMap<String, Vec<TrendPoint>>,
}

impl MarketPredictor {
    /// Create a new market predictor
    pub fn new(config: PredictorConfig) -> Self {
        Self {
            config,
            keyword_trends: HashMap::new(),
        }
    }

    /// Create with default configuration
    pub fn default_predictor() -> Self {
        Self::new(PredictorConfig::default())
    }

    /// Analyze market from opportunities
    pub fn analyze_from_opportunities(
        &self,
        opportunities: &[Opportunity],
    ) -> MarketAnalysisResult {
        info!("Analyzing market from {} opportunities", opportunities.len());

        // Extract keywords and categories
        let mut keyword_counts: HashMap<String, usize> = HashMap::new();
        let mut source_distribution: HashMap<OpportunitySource, usize> = HashMap::new();

        for opp in opportunities {
            // Count sources
            *source_distribution.entry(opp.source).or_insert(0) += 1;

            // Extract keywords
            let text = format!("{} {}", opp.title, opp.description).to_lowercase();
            for keyword in self.extract_keywords(&text) {
                *keyword_counts.entry(keyword).or_insert(0) += 1;
            }
        }

        // Identify hot topics
        let mut hot_topics: Vec<_> = keyword_counts.into_iter().collect();
        hot_topics.sort_by(|a, b| b.1.cmp(&a.1));
        let hot_topics: Vec<_> = hot_topics.into_iter().take(10).collect();

        // Calculate average score
        let avg_score: f64 = opportunities
            .iter()
            .filter_map(|o| o.score)
            .sum::<f64>()
            / opportunities.len().max(1) as f64;

        // Determine overall trend
        let trend = self.determine_trend(opportunities);

        // Generate market segments
        let segments = self.identify_segments(opportunities);

        // Generate predictions
        let predictions = self.generate_predictions(&hot_topics, opportunities);

        MarketAnalysisResult {
            total_opportunities: opportunities.len(),
            average_score: avg_score,
            overall_trend: trend,
            hot_topics,
            source_distribution: source_distribution
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            segments,
            predictions,
            insights: self.generate_insights(opportunities),
        }
    }

    /// Extract keywords from text
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        let tech_keywords = [
            "ai", "api", "automation", "saas", "analytics", "dashboard", "mobile",
            "web", "cloud", "integration", "platform", "tool", "service", "data",
            "machine learning", "crypto", "blockchain", "defi", "nft", "workflow",
            "productivity", "marketing", "sales", "crm", "email", "social",
            "developer", "devtools", "low-code", "no-code", "fintech", "healthtech",
        ];

        let mut found = Vec::new();
        for keyword in tech_keywords {
            if text.contains(keyword) {
                found.push(keyword.to_string());
            }
        }
        found
    }

    /// Determine overall market trend
    fn determine_trend(&self, opportunities: &[Opportunity]) -> MarketTrend {
        if opportunities.len() < self.config.min_data_points {
            return MarketTrend::Stable;
        }

        // Sort by creation date
        let mut sorted = opportunities.to_vec();
        sorted.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        // Split into halves
        let mid = sorted.len() / 2;
        let (first_half, second_half) = sorted.split_at(mid);

        // Compare average scores
        let first_avg: f64 = first_half
            .iter()
            .filter_map(|o| o.score)
            .sum::<f64>()
            / first_half.len().max(1) as f64;

        let second_avg: f64 = second_half
            .iter()
            .filter_map(|o| o.score)
            .sum::<f64>()
            / second_half.len().max(1) as f64;

        // Compare volumes
        let volume_change = (second_half.len() as f64 - first_half.len() as f64)
            / first_half.len().max(1) as f64;

        if volume_change > 0.2 && second_avg > first_avg {
            MarketTrend::Rising
        } else if volume_change < -0.2 && second_avg < first_avg {
            MarketTrend::Declining
        } else if (volume_change.abs() > 0.3) || (second_avg - first_avg).abs() > 20.0 {
            MarketTrend::Volatile
        } else {
            MarketTrend::Stable
        }
    }

    /// Identify market segments
    fn identify_segments(&self, opportunities: &[Opportunity]) -> Vec<MarketSegment> {
        let mut segments = Vec::new();

        // Define segment patterns
        let segment_patterns = [
            ("SaaS/Productivity", vec!["saas", "productivity", "workflow", "tool"]),
            ("AI/ML", vec!["ai", "machine learning", "automation", "analytics"]),
            ("Developer Tools", vec!["developer", "api", "devtools", "integration"]),
            ("Marketing/Sales", vec!["marketing", "sales", "crm", "email"]),
            ("Fintech", vec!["fintech", "payment", "crypto", "blockchain"]),
        ];

        for (segment_name, keywords) in segment_patterns {
            let matching: Vec<_> = opportunities
                .iter()
                .filter(|o| {
                    let text = format!("{} {}", o.title, o.description).to_lowercase();
                    keywords.iter().any(|k| text.contains(k))
                })
                .collect();

            if !matching.is_empty() {
                let avg_score: f64 = matching
                    .iter()
                    .filter_map(|o| o.score)
                    .sum::<f64>()
                    / matching.len() as f64;

                segments.push(MarketSegment {
                    name: segment_name.to_string(),
                    size_estimate: self.estimate_segment_size(matching.len()),
                    growth_rate: (avg_score - 50.0) / 100.0, // Simplified growth estimate
                    key_players: vec![],
                    entry_barriers: self.identify_barriers(segment_name),
                    opportunities: matching
                        .iter()
                        .take(3)
                        .map(|o| o.title.clone())
                        .collect(),
                });
            }
        }

        segments
    }

    /// Estimate segment size based on opportunity count
    fn estimate_segment_size(&self, count: usize) -> String {
        if count > 20 {
            "Large (High demand)".to_string()
        } else if count > 10 {
            "Medium (Growing)".to_string()
        } else {
            "Small (Niche)".to_string()
        }
    }

    /// Identify entry barriers for a segment
    fn identify_barriers(&self, segment: &str) -> Vec<String> {
        match segment {
            "AI/ML" => vec![
                "Technical expertise required".to_string(),
                "High compute costs".to_string(),
                "Data requirements".to_string(),
            ],
            "Fintech" => vec![
                "Regulatory compliance".to_string(),
                "Security requirements".to_string(),
                "Trust building".to_string(),
            ],
            "SaaS/Productivity" => vec![
                "Crowded market".to_string(),
                "Customer acquisition costs".to_string(),
            ],
            _ => vec!["General competition".to_string()],
        }
    }

    /// Generate predictions for hot topics
    fn generate_predictions(
        &self,
        hot_topics: &[(String, usize)],
        opportunities: &[Opportunity],
    ) -> Vec<MarketPrediction> {
        hot_topics
            .iter()
            .take(5)
            .map(|(topic, count)| {
                let relevance = *count as f64 / opportunities.len().max(1) as f64;

                MarketPrediction {
                    category: topic.clone(),
                    trend: if relevance > 0.3 {
                        MarketTrend::Rising
                    } else {
                        MarketTrend::Stable
                    },
                    confidence: ConfidenceLevel::from_score(relevance),
                    growth_potential: relevance * 100.0,
                    competition_intensity: self.estimate_competition(topic),
                    entry_difficulty: self.estimate_entry_difficulty(topic),
                    key_factors: self.identify_key_factors(topic),
                    recommended_actions: self.recommend_actions(topic, relevance),
                }
            })
            .collect()
    }

    /// Estimate competition intensity
    fn estimate_competition(&self, topic: &str) -> f64 {
        match topic {
            "ai" | "automation" => 0.8,
            "saas" | "dashboard" => 0.7,
            "api" | "integration" => 0.6,
            _ => 0.5,
        }
    }

    /// Estimate entry difficulty
    fn estimate_entry_difficulty(&self, topic: &str) -> f64 {
        match topic {
            "ai" | "machine learning" => 0.8,
            "blockchain" | "crypto" => 0.7,
            "saas" | "tool" => 0.4,
            _ => 0.5,
        }
    }

    /// Identify key success factors
    fn identify_key_factors(&self, topic: &str) -> Vec<String> {
        match topic {
            "ai" | "automation" => vec![
                "Quality of AI models".to_string(),
                "User experience".to_string(),
                "Integration capabilities".to_string(),
            ],
            "saas" => vec![
                "Pricing strategy".to_string(),
                "Feature differentiation".to_string(),
                "Customer support".to_string(),
            ],
            _ => vec![
                "Product-market fit".to_string(),
                "Go-to-market strategy".to_string(),
            ],
        }
    }

    /// Recommend actions based on topic and relevance
    fn recommend_actions(&self, topic: &str, relevance: f64) -> Vec<String> {
        let mut actions = Vec::new();

        if relevance > 0.3 {
            actions.push(format!("High demand for {} - consider prioritizing", topic));
        }

        match topic {
            "ai" | "automation" => {
                actions.push("Focus on specific use cases rather than generic AI".to_string());
                actions.push("Consider partnerships with AI providers".to_string());
            }
            "saas" => {
                actions.push("Identify underserved niches within SaaS".to_string());
                actions.push("Focus on specific pain points".to_string());
            }
            _ => {
                actions.push(format!("Research {} market further", topic));
            }
        }

        actions
    }

    /// Generate market insights
    fn generate_insights(&self, opportunities: &[Opportunity]) -> Vec<String> {
        let mut insights = Vec::new();

        let total = opportunities.len();
        if total == 0 {
            return vec!["Not enough data to generate insights".to_string()];
        }

        // Source distribution insight
        let github_count = opportunities
            .iter()
            .filter(|o| matches!(o.source, OpportunitySource::GitHub))
            .count();

        if github_count as f64 / total as f64 > 0.4 {
            insights.push("Strong developer-focused demand detected from GitHub trends".to_string());
        }

        // Score distribution insight
        let high_score = opportunities.iter().filter(|o| o.score.unwrap_or(0.0) > 80.0).count();
        if high_score as f64 / total as f64 > 0.2 {
            insights.push(format!(
                "{}% of opportunities have high potential (score > 80)",
                (high_score * 100 / total)
            ));
        }

        // Pain point insight
        let with_pain_points = opportunities.iter().filter(|o| !o.pain_points.is_empty()).count();
        if with_pain_points > 0 {
            insights.push(format!(
                "{} opportunities have clearly identified pain points",
                with_pain_points
            ));
        }

        insights
    }

    /// Update trend data
    pub fn update_trend(&mut self, keyword: &str, score: f64) {
        let trends = self.keyword_trends.entry(keyword.to_string()).or_default();
        trends.push(TrendPoint {
            timestamp: chrono::Utc::now(),
            score,
        });

        // Keep only recent data
        let cutoff = chrono::Utc::now() - chrono::Duration::days(self.config.lookback_days as i64);
        trends.retain(|t| t.timestamp > cutoff);
    }
}

/// A single trend data point
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TrendPoint {
    timestamp: chrono::DateTime<chrono::Utc>,
    score: f64,
}

/// Complete market analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketAnalysisResult {
    pub total_opportunities: usize,
    pub average_score: f64,
    pub overall_trend: MarketTrend,
    pub hot_topics: Vec<(String, usize)>,
    pub source_distribution: HashMap<String, usize>,
    pub segments: Vec<MarketSegment>,
    pub predictions: Vec<MarketPrediction>,
    pub insights: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_opportunities() -> Vec<Opportunity> {
        vec![
            {
                let mut o = Opportunity::new(
                    "AI-powered SaaS tool",
                    "Automation platform for workflows",
                    OpportunitySource::GitHub,
                );
                o.set_score(85.0);
                o
            },
            {
                let mut o = Opportunity::new(
                    "Developer API integration",
                    "REST API for developers",
                    OpportunitySource::Reddit,
                );
                o.set_score(75.0);
                o
            },
        ]
    }

    #[test]
    fn test_market_predictor_creation() {
        let predictor = MarketPredictor::default_predictor();
        assert_eq!(predictor.config.min_data_points, 5);
    }

    #[test]
    fn test_keyword_extraction() {
        let predictor = MarketPredictor::default_predictor();
        let keywords = predictor.extract_keywords("ai-powered saas automation tool");

        assert!(keywords.contains(&"ai".to_string()));
        assert!(keywords.contains(&"saas".to_string()));
        assert!(keywords.contains(&"automation".to_string()));
    }

    #[test]
    fn test_market_analysis() {
        let predictor = MarketPredictor::default_predictor();
        let opportunities = create_test_opportunities();

        let result = predictor.analyze_from_opportunities(&opportunities);

        assert_eq!(result.total_opportunities, 2);
        assert!(result.average_score > 0.0);
        assert!(!result.hot_topics.is_empty());
    }

    #[test]
    fn test_confidence_level() {
        assert_eq!(ConfidenceLevel::from_score(0.9), ConfidenceLevel::High);
        assert_eq!(ConfidenceLevel::from_score(0.6), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_score(0.3), ConfidenceLevel::Low);
    }
}

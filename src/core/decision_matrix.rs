//! Decision Matrix - AI-powered opportunity evaluation

use crate::models::{Decision, Opportunity, Portfolio, RankedOpportunity, TemplateType};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Configuration for the decision matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixConfig {
    /// Minimum score threshold for launch consideration
    pub min_launch_score: f64,
    /// Maximum concurrent projects allowed
    pub max_concurrent_projects: u32,
    /// Budget per project in USD
    pub budget_per_project: f64,
    /// Risk tolerance (0-1, higher = more risk-tolerant)
    pub risk_tolerance: f64,
    /// Prefer diversification over concentration
    pub prefer_diversification: bool,
}

impl Default for MatrixConfig {
    fn default() -> Self {
        Self {
            min_launch_score: 70.0,
            max_concurrent_projects: 5,
            budget_per_project: 500.0,
            risk_tolerance: 0.5,
            prefer_diversification: true,
        }
    }
}

/// Evaluation criteria weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationWeights {
    pub market_demand: f64,
    pub competition: f64,
    pub technical_feasibility: f64,
    pub revenue_potential: f64,
    pub strategic_fit: f64,
}

impl Default for EvaluationWeights {
    fn default() -> Self {
        Self {
            market_demand: 0.25,
            competition: 0.15,
            technical_feasibility: 0.20,
            revenue_potential: 0.25,
            strategic_fit: 0.15,
        }
    }
}

/// Decision matrix for evaluating and ranking opportunities
pub struct DecisionMatrix {
    config: MatrixConfig,
    weights: EvaluationWeights,
}

impl DecisionMatrix {
    /// Create a new decision matrix
    pub fn new(config: MatrixConfig) -> Self {
        Self {
            config,
            weights: EvaluationWeights::default(),
        }
    }

    /// Create with default configuration
    pub fn default_matrix() -> Self {
        Self::new(MatrixConfig::default())
    }

    /// Set custom evaluation weights
    pub fn with_weights(mut self, weights: EvaluationWeights) -> Self {
        self.weights = weights;
        self
    }

    /// Evaluate and rank a list of opportunities
    pub fn evaluate_opportunities(
        &self,
        opportunities: Vec<Opportunity>,
        current_portfolio: &Portfolio,
    ) -> Vec<RankedOpportunity> {
        info!(
            "Evaluating {} opportunities against portfolio of {} projects",
            opportunities.len(),
            current_portfolio.total_projects
        );

        let mut ranked: Vec<RankedOpportunity> = opportunities
            .into_iter()
            .filter_map(|opp| {
                let evaluation = self.evaluate_single(&opp, current_portfolio);
                if evaluation.total_score >= self.config.min_launch_score * 0.5 {
                    Some(RankedOpportunity {
                        opportunity: opp,
                        rank: 0, // Will be set after sorting
                        reasoning: evaluation.reasoning,
                        confidence: evaluation.confidence,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by evaluation score (highest first)
        ranked.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Assign ranks
        for (i, ranked_opp) in ranked.iter_mut().enumerate() {
            ranked_opp.rank = (i + 1) as u32;
        }

        ranked
    }

    /// Evaluate a single opportunity
    fn evaluate_single(&self, opp: &Opportunity, portfolio: &Portfolio) -> OpportunityEvaluation {
        let mut scores = EvaluationScores::default();
        let mut reasoning_parts = Vec::new();

        // Market demand score
        scores.market_demand = self.score_market_demand(opp);
        reasoning_parts.push(format!(
            "Market demand: {:.1}/100",
            scores.market_demand
        ));

        // Competition score
        scores.competition = self.score_competition(opp);
        reasoning_parts.push(format!(
            "Competition favorability: {:.1}/100",
            scores.competition
        ));

        // Technical feasibility score
        scores.technical_feasibility = self.score_technical_feasibility(opp);
        reasoning_parts.push(format!(
            "Technical feasibility: {:.1}/100",
            scores.technical_feasibility
        ));

        // Revenue potential score
        scores.revenue_potential = self.score_revenue_potential(opp);
        reasoning_parts.push(format!(
            "Revenue potential: {:.1}/100",
            scores.revenue_potential
        ));

        // Strategic fit score
        scores.strategic_fit = self.score_strategic_fit(opp, portfolio);
        reasoning_parts.push(format!(
            "Strategic fit: {:.1}/100",
            scores.strategic_fit
        ));

        // Calculate weighted total
        let total_score = scores.market_demand * self.weights.market_demand
            + scores.competition * self.weights.competition
            + scores.technical_feasibility * self.weights.technical_feasibility
            + scores.revenue_potential * self.weights.revenue_potential
            + scores.strategic_fit * self.weights.strategic_fit;

        // Calculate confidence based on data completeness
        let confidence = self.calculate_confidence(opp, total_score);

        OpportunityEvaluation {
            scores,
            total_score,
            confidence,
            reasoning: reasoning_parts.join("; "),
        }
    }

    /// Score market demand based on engagement signals
    fn score_market_demand(&self, opp: &Opportunity) -> f64 {
        let mut score = opp.score.unwrap_or(50.0);

        // Boost from upvotes
        if let Some(upvotes) = opp.raw_data.upvotes {
            if upvotes > 1000 {
                score += 20.0;
            } else if upvotes > 100 {
                score += 10.0;
            } else if upvotes > 10 {
                score += 5.0;
            }
        }

        // Boost from comments (indicates discussion/interest)
        if let Some(comments) = opp.raw_data.comments_count {
            if comments > 100 {
                score += 15.0;
            } else if comments > 20 {
                score += 8.0;
            } else if comments > 5 {
                score += 3.0;
            }
        }

        // Boost from pain points
        score += (opp.pain_points.len() as f64) * 5.0;

        score.clamp(0.0, 100.0)
    }

    /// Score competition level (higher = less competition = better)
    fn score_competition(&self, opp: &Opportunity) -> f64 {
        // Start with a neutral score
        let mut score: f64 = 60.0;

        // Check market analysis if available
        if let Some(ref market) = opp.market_analysis {
            match market.competition_level.to_lowercase().as_str() {
                "low" | "minimal" => score = 85.0,
                "medium" | "moderate" => score = 60.0,
                "high" | "intense" => score = 35.0,
                _ => {}
            }
        }

        // Check title/description for competition signals
        let text = format!("{} {}", opp.title, opp.description).to_lowercase();

        // Niche indicators (positive)
        let niche_keywords = ["niche", "underserved", "gap", "untapped", "unique"];
        for keyword in niche_keywords {
            if text.contains(keyword) {
                score += 10.0;
            }
        }

        // Crowded market indicators (negative)
        let crowded_keywords = ["competitor", "alternative to", "like", "similar to"];
        for keyword in crowded_keywords {
            if text.contains(keyword) {
                score -= 5.0;
            }
        }

        score.clamp(0.0, 100.0)
    }

    /// Score technical feasibility
    fn score_technical_feasibility(&self, opp: &Opportunity) -> f64 {
        let mut score = 70.0; // Default to moderately feasible

        // Check technical analysis if available
        if let Some(ref tech) = opp.technical_analysis {
            // Lower complexity = higher score
            score = 100.0 - (tech.complexity_score * 0.8);

            // Adjust based on estimated hours
            if tech.estimated_hours < 40 {
                score += 10.0;
            } else if tech.estimated_hours > 160 {
                score -= 15.0;
            }
        }

        // Analyze description for complexity signals
        let text = opp.description.to_lowercase();

        // Simple/achievable indicators
        let simple_keywords = ["simple", "basic", "mvp", "landing page", "crud", "api"];
        for keyword in simple_keywords {
            if text.contains(keyword) {
                score += 5.0;
            }
        }

        // Complex indicators
        let complex_keywords = [
            "machine learning",
            "blockchain",
            "real-time",
            "distributed",
            "enterprise",
        ];
        for keyword in complex_keywords {
            if text.contains(keyword) {
                score -= 10.0;
            }
        }

        score.clamp(0.0, 100.0)
    }

    /// Score revenue potential
    fn score_revenue_potential(&self, opp: &Opportunity) -> f64 {
        let mut score = 50.0;

        // Check monetization analysis if available
        if let Some(ref mon) = opp.monetization_analysis {
            score = mon.clarity_score;

            // Boost from clear price range
            if mon.price_range.is_some() {
                score += 10.0;
            }

            // Boost from MRR potential
            if let Some(mrr) = mon.potential_mrr {
                if mrr > 10000.0 {
                    score += 20.0;
                } else if mrr > 1000.0 {
                    score += 10.0;
                }
            }
        }

        // Analyze for monetization keywords
        let text = format!("{} {}", opp.title, opp.description).to_lowercase();

        let monetization_keywords = [
            "saas",
            "subscription",
            "premium",
            "paid",
            "pricing",
            "revenue",
            "monetize",
            "business",
        ];

        for keyword in monetization_keywords {
            if text.contains(keyword) {
                score += 5.0;
            }
        }

        // Check template type alignment
        let text_lower = text.to_lowercase();
        if text_lower.contains("api") || text_lower.contains("service") {
            score += 5.0; // APIs are easier to monetize
        }

        score.clamp(0.0, 100.0)
    }

    /// Score strategic fit with current portfolio
    fn score_strategic_fit(&self, opp: &Opportunity, portfolio: &Portfolio) -> f64 {
        let mut score: f64 = 70.0;

        // Check portfolio diversification
        if self.config.prefer_diversification {
            // Bonus for different sources than existing projects
            let existing_sources: std::collections::HashSet<_> = portfolio
                .projects
                .iter()
                .filter_map(|p| p.opportunity_id.as_ref())
                .collect();

            if !existing_sources.contains(&opp.id) {
                score += 10.0;
            }
        }

        // Penalty if at capacity
        if portfolio.active_projects >= self.config.max_concurrent_projects as u64 {
            score -= 30.0;
        }

        // Bonus for synergies with existing projects
        let opp_text = format!("{} {}", opp.title, opp.description).to_lowercase();
        for project in &portfolio.projects {
            let project_text = format!("{} {}", project.name, project.description).to_lowercase();

            // Check for keyword overlap (potential synergy)
            let opp_words: std::collections::HashSet<_> = opp_text.split_whitespace().collect();
            let project_words: std::collections::HashSet<_> =
                project_text.split_whitespace().collect();

            let overlap = opp_words.intersection(&project_words).count();
            if overlap > 3 {
                score += 5.0; // Some synergy
            }
        }

        score.clamp(0.0, 100.0)
    }

    /// Calculate confidence in the evaluation
    fn calculate_confidence(&self, opp: &Opportunity, total_score: f64) -> f64 {
        let mut confidence = total_score;

        // Boost confidence if we have detailed analysis
        if opp.market_analysis.is_some() {
            confidence += 5.0;
        }
        if opp.technical_analysis.is_some() {
            confidence += 5.0;
        }
        if opp.monetization_analysis.is_some() {
            confidence += 5.0;
        }

        // Reduce confidence for very new opportunities (might be noise)
        let age_hours = (chrono::Utc::now() - opp.created_at).num_hours();
        if age_hours < 24 {
            confidence -= 10.0;
        }

        confidence.clamp(0.0, 100.0)
    }

    /// Decide whether to launch a specific opportunity
    pub fn should_launch(&self, opp: &Opportunity, portfolio: &Portfolio) -> Decision {
        debug!("Evaluating launch decision for opportunity: {}", opp.id);

        let evaluation = self.evaluate_single(opp, portfolio);

        // Check capacity
        if portfolio.active_projects >= self.config.max_concurrent_projects as u64 {
            return Decision::defer(
                "Portfolio at maximum capacity",
                7, // Revisit in a week
            );
        }

        // Check score threshold
        if evaluation.total_score < self.config.min_launch_score {
            if evaluation.total_score >= self.config.min_launch_score * 0.7 {
                return Decision::defer(
                    format!(
                        "Score {:.1} below threshold {:.1}, but close",
                        evaluation.total_score, self.config.min_launch_score
                    ),
                    14,
                );
            } else {
                return Decision::reject(format!(
                    "Score {:.1} significantly below threshold {:.1}",
                    evaluation.total_score, self.config.min_launch_score
                ));
            }
        }

        // Suggest template type based on opportunity
        let template = self.suggest_template(opp);

        Decision::Launch {
            confidence: evaluation.confidence,
            reasoning: evaluation.reasoning,
            suggested_template: template,
        }
    }

    /// Suggest the best template type for an opportunity
    fn suggest_template(&self, opp: &Opportunity) -> TemplateType {
        let text = format!("{} {}", opp.title, opp.description).to_lowercase();

        if text.contains("api") || text.contains("backend") || text.contains("service") {
            TemplateType::ApiService
        } else if text.contains("blog")
            || text.contains("content")
            || text.contains("seo")
            || text.contains("article")
        {
            TemplateType::ContentSite
        } else if text.contains("extension") || text.contains("chrome") || text.contains("browser")
        {
            TemplateType::ChromeExtension
        } else if text.contains("cli") || text.contains("command line") || text.contains("terminal")
        {
            TemplateType::CliTool
        } else if text.contains("app") && (text.contains("mobile") || text.contains("ios") || text.contains("android"))
        {
            TemplateType::MobileApp
        } else {
            // Default to SaaS
            TemplateType::MicroSaas
        }
    }
}

/// Internal evaluation scores
#[derive(Debug, Default)]
struct EvaluationScores {
    market_demand: f64,
    competition: f64,
    technical_feasibility: f64,
    revenue_potential: f64,
    strategic_fit: f64,
}

/// Internal evaluation result
#[derive(Debug)]
struct OpportunityEvaluation {
    #[allow(dead_code)]
    scores: EvaluationScores,
    total_score: f64,
    confidence: f64,
    reasoning: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::OpportunitySource;

    fn create_test_opportunity() -> Opportunity {
        let mut opp = Opportunity::new(
            "SaaS Dashboard for Analytics",
            "A subscription-based tool for business analytics and reporting",
            OpportunitySource::Manual,
        );
        opp.set_score(75.0);
        opp.raw_data.upvotes = Some(150);
        opp.raw_data.comments_count = Some(25);
        opp
    }

    #[test]
    fn test_decision_matrix_creation() {
        let matrix = DecisionMatrix::default_matrix();
        assert!(matrix.config.min_launch_score > 0.0);
        assert!(matrix.config.max_concurrent_projects > 0);
    }

    #[test]
    fn test_evaluate_opportunities() {
        let matrix = DecisionMatrix::default_matrix();
        let portfolio = Portfolio::new();
        let opp = create_test_opportunity();

        let ranked = matrix.evaluate_opportunities(vec![opp], &portfolio);
        assert!(!ranked.is_empty());
        assert_eq!(ranked[0].rank, 1);
    }

    #[test]
    fn test_should_launch_decision() {
        let matrix = DecisionMatrix::default_matrix();
        let portfolio = Portfolio::new();
        let opp = create_test_opportunity();

        let decision = matrix.should_launch(&opp, &portfolio);
        // With good engagement signals, should recommend launch
        assert!(decision.is_launch());
    }

    #[test]
    fn test_template_suggestion() {
        let matrix = DecisionMatrix::default_matrix();

        let api_opp = Opportunity::new(
            "REST API Service",
            "A backend API service for data processing",
            OpportunitySource::Manual,
        );
        assert_eq!(matrix.suggest_template(&api_opp), TemplateType::ApiService);

        let saas_opp = Opportunity::new(
            "Project Management Tool",
            "A SaaS platform for team collaboration",
            OpportunitySource::Manual,
        );
        assert_eq!(matrix.suggest_template(&saas_opp), TemplateType::MicroSaas);
    }

    #[test]
    fn test_capacity_check() {
        let matrix = DecisionMatrix::new(MatrixConfig {
            max_concurrent_projects: 1,
            ..Default::default()
        });

        let mut portfolio = Portfolio::new();
        portfolio.active_projects = 1; // At capacity

        let opp = create_test_opportunity();
        let decision = matrix.should_launch(&opp, &portfolio);

        // Should defer due to capacity
        assert!(matches!(decision, Decision::Defer { .. }));
    }
}

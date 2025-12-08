//! Integration tests for opportunity scanner

use genesis_engine::core::{OpportunityScanner, ScannerConfig};
use genesis_engine::db::Database;
use genesis_engine::models::{Opportunity, OpportunitySource};
use std::sync::Arc;

fn create_test_scanner() -> OpportunityScanner {
    let db = Arc::new(Database::in_memory().unwrap());
    OpportunityScanner::new(ScannerConfig::default(), db)
}

#[test]
fn test_scanner_creation() {
    let scanner = create_test_scanner();
    // Scanner should be created without errors
    assert!(scanner.http_client().timeout().is_some());
}

#[test]
fn test_opportunity_scoring() {
    let scanner = create_test_scanner();
    
    let mut opp = Opportunity::new(
        "AI SaaS Automation Tool",
        "A powerful tool to automate workflows with AI-powered analytics",
        OpportunitySource::Manual,
    );
    
    // Initial score should be None
    assert!(opp.score.is_none());
    
    // Score the opportunity
    scanner.score_opportunity(&mut opp);
    
    // Score should now be set
    assert!(opp.score.is_some());
    
    // Score should be boosted due to keywords (ai, saas, automation, analytics)
    assert!(opp.score.unwrap() > 50.0);
}

#[test]
fn test_negative_keyword_scoring() {
    let scanner = create_test_scanner();
    
    let mut opp = Opportunity::new(
        "Free Homework Helper",
        "A free tool to help with homework assignments",
        OpportunitySource::Manual,
    );
    
    scanner.score_opportunity(&mut opp);
    
    // Score should be lowered due to negative keywords (free, homework, assignment)
    assert!(opp.score.unwrap() < 50.0);
}

#[test]
fn test_engagement_boost() {
    let scanner = create_test_scanner();
    
    let mut low_engagement = Opportunity::new(
        "Test Project",
        "A simple test",
        OpportunitySource::Manual,
    );
    low_engagement.raw_data.upvotes = Some(1);
    low_engagement.raw_data.comments_count = Some(0);
    
    let mut high_engagement = Opportunity::new(
        "Test Project",
        "A simple test",
        OpportunitySource::Manual,
    );
    high_engagement.raw_data.upvotes = Some(1000);
    high_engagement.raw_data.comments_count = Some(500);
    
    scanner.score_opportunity(&mut low_engagement);
    scanner.score_opportunity(&mut high_engagement);
    
    // High engagement should score higher
    assert!(high_engagement.score.unwrap() > low_engagement.score.unwrap());
}

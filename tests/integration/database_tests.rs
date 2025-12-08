//! Integration tests for database operations

use genesis_engine::db::Database;
use genesis_engine::models::{
    Deployment, Opportunity, OpportunitySource, Platform, Project, ProjectStatus,
};

fn create_test_db() -> Database {
    Database::in_memory().unwrap()
}

#[test]
fn test_database_initialization() {
    let db = create_test_db();
    // Should succeed without errors
    assert!(db.initialize().is_ok());
}

#[test]
fn test_opportunity_crud() {
    let db = create_test_db();
    db.initialize().unwrap();

    // Create
    let mut opp = Opportunity::new(
        "Test Opportunity",
        "A test description",
        OpportunitySource::GitHub,
    );
    opp.set_score(85.0);

    db.save_opportunity(&opp).unwrap();

    // Read
    let loaded = db.get_opportunity(&opp.id).unwrap().unwrap();
    assert_eq!(loaded.title, "Test Opportunity");
    assert_eq!(loaded.score, Some(85.0));

    // Update
    let mut updated = loaded.clone();
    updated.set_score(90.0);
    db.save_opportunity(&updated).unwrap();

    let reloaded = db.get_opportunity(&opp.id).unwrap().unwrap();
    assert_eq!(reloaded.score, Some(90.0));
}

#[test]
fn test_project_crud() {
    let db = create_test_db();
    db.initialize().unwrap();

    // Create
    let project = Project::new("Test Project", "A test project");
    db.save_project(&project).unwrap();

    // Read
    let loaded = db.get_project(&project.id).unwrap().unwrap();
    assert_eq!(loaded.name, "Test Project");
    assert_eq!(loaded.status, ProjectStatus::Initializing);

    // Update status
    let mut updated = loaded.clone();
    updated.set_status(ProjectStatus::Live);
    db.save_project(&updated).unwrap();

    let reloaded = db.get_project(&project.id).unwrap().unwrap();
    assert_eq!(reloaded.status, ProjectStatus::Live);
    assert!(reloaded.launched_at.is_some());
}

#[test]
fn test_deployment_crud() {
    let db = create_test_db();
    db.initialize().unwrap();

    // First create a project
    let project = Project::new("Test Project", "A test project");
    db.save_project(&project).unwrap();

    // Create deployment
    let mut deployment = Deployment::new(&project.id, Platform::Vercel);
    deployment.set_url("https://test.vercel.app");
    db.save_deployment(&deployment).unwrap();

    // Read
    let loaded = db.get_deployment(&deployment.id).unwrap().unwrap();
    assert_eq!(loaded.project_id, project.id);
    assert_eq!(loaded.platform, Platform::Vercel);
    assert_eq!(loaded.url.unwrap(), "https://test.vercel.app");
}

#[test]
fn test_list_opportunities_with_filters() {
    let db = create_test_db();
    db.initialize().unwrap();

    // Create multiple opportunities
    let mut opp1 = Opportunity::new("High Score", "Test", OpportunitySource::GitHub);
    opp1.set_score(90.0);
    db.save_opportunity(&opp1).unwrap();

    let mut opp2 = Opportunity::new("Low Score", "Test", OpportunitySource::Reddit);
    opp2.set_score(40.0);
    db.save_opportunity(&opp2).unwrap();

    let mut opp3 = Opportunity::new("Medium Score", "Test", OpportunitySource::GitHub);
    opp3.set_score(60.0);
    db.save_opportunity(&opp3).unwrap();

    // Filter by min score
    let high_scorers = db.list_opportunities(None, None, Some(70.0), None).unwrap();
    assert_eq!(high_scorers.len(), 1);
    assert_eq!(high_scorers[0].title, "High Score");

    // Filter by source
    let github = db.list_opportunities(None, Some("github"), None, None).unwrap();
    assert_eq!(github.len(), 2);

    // Limit results
    let limited = db.list_opportunities(None, None, None, Some(2)).unwrap();
    assert_eq!(limited.len(), 2);
}

#[test]
fn test_list_projects() {
    let db = create_test_db();
    db.initialize().unwrap();

    // Create projects with different statuses
    let mut live1 = Project::new("Live 1", "Test");
    live1.status = ProjectStatus::Live;
    db.save_project(&live1).unwrap();

    let mut live2 = Project::new("Live 2", "Test");
    live2.status = ProjectStatus::Live;
    db.save_project(&live2).unwrap();

    let failed = Project::new("Failed", "Test");
    db.save_project(&failed).unwrap();

    // Filter by status
    let live_projects = db.list_projects(Some("live"), None).unwrap();
    assert_eq!(live_projects.len(), 2);

    // Get all
    let all_projects = db.list_projects(None, None).unwrap();
    assert_eq!(all_projects.len(), 3);
}

#[test]
fn test_portfolio_stats() {
    let db = create_test_db();
    db.initialize().unwrap();

    // Create some projects
    let mut project1 = Project::new("Project 1", "Test");
    project1.status = ProjectStatus::Live;
    project1.total_revenue_usd = 100.0;
    project1.active_users = 50;
    db.save_project(&project1).unwrap();

    let mut project2 = Project::new("Project 2", "Test");
    project2.status = ProjectStatus::Live;
    project2.total_revenue_usd = 200.0;
    project2.active_users = 100;
    db.save_project(&project2).unwrap();

    let mut project3 = Project::new("Project 3", "Test");
    project3.status = ProjectStatus::Failed;
    db.save_project(&project3).unwrap();

    // Get stats
    let stats = db.get_portfolio_stats().unwrap();

    assert_eq!(stats.total_projects, 3);
    assert_eq!(stats.live_projects, 2);
    assert_eq!(stats.failed_projects, 1);
    assert_eq!(stats.total_revenue, 300.0);
    assert_eq!(stats.total_users, 150);
    assert_eq!(stats.avg_revenue_per_project, 150.0);
}

#[test]
fn test_deployments_for_project() {
    let db = create_test_db();
    db.initialize().unwrap();

    // Create a project
    let project = Project::new("Test Project", "Test");
    db.save_project(&project).unwrap();

    // Create multiple deployments
    for i in 0..3 {
        let mut deployment = Deployment::new(&project.id, Platform::Vercel);
        deployment.set_url(format!("https://test-{}.vercel.app", i));
        db.save_deployment(&deployment).unwrap();
    }

    // List deployments
    let deployments = db.list_deployments_for_project(&project.id).unwrap();
    assert_eq!(deployments.len(), 3);
}

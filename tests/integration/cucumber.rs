use cucumber::{given, then, when, World};
use std::sync::Arc;
use tokio::sync::Mutex;
use tempfile::tempdir;

use toggl::{
    models::{Project, TimeEntry, Timer},
    repository::{Repository, SqliteRepository},
    services::{ProjectService, TimeTrackingService},
};

#[derive(Debug, World)]
pub struct TogglWorld {
    pub repository: Arc<dyn Repository>,
    pub project_service: ProjectService,
    pub tracking_service: TimeTrackingService,
    pub current_output: Option<String>,
    pub current_error: Option<String>,
    pub active_timer: Option<Timer>,
}

impl Default for TogglWorld {
    fn default() -> Self {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let repository = Arc::new(SqliteRepository::new(db_path.to_str().unwrap()).unwrap());
        
        let project_service = ProjectService::new(repository.clone());
        let tracking_service = TimeTrackingService::new(repository.clone());

        Self {
            repository,
            project_service,
            tracking_service,
            current_output: None,
            current_error: None,
            active_timer: None,
        }
    }
}

// Background steps
#[given("a clean toggl database")]
async fn clean_database(world: &mut TogglWorld) {
    world.repository.clear_all().await.unwrap();
}

#[given(regex = r#"I have a project called "([^"]*)""#)]
async fn create_project(world: &mut TogglWorld, project_name: String) {
    world
        .project_service
        .create_project(&project_name, None)
        .await
        .unwrap();
}

// Time tracking steps
#[when(regex = r#"I start tracking time for project "([^"]*)""#)]
async fn start_tracking(world: &mut TogglWorld, project_name: String) {
    match world.tracking_service.start_timer(&project_name, None).await {
        Ok(timer) => world.active_timer = Some(timer),
        Err(e) => world.current_error = Some(e.to_string()),
    }
}

#[when(regex = r#"I start tracking time for project "([^"]*)" with task "([^"]*)""#)]
async fn start_tracking_with_task(world: &mut TogglWorld, project_name: String, task: String) {
    match world
        .tracking_service
        .start_timer(&project_name, Some(&task))
        .await
    {
        Ok(timer) => world.active_timer = Some(timer),
        Err(e) => world.current_error = Some(e.to_string()),
    }
}

#[when("I stop tracking time")]
async fn stop_tracking(world: &mut TogglWorld) {
    match world.tracking_service.stop_timer().await {
        Ok(_) => world.active_timer = None,
        Err(e) => world.current_error = Some(e.to_string()),
    }
}

#[when("I check the current status")]
async fn check_status(world: &mut TogglWorld) {
    match world.tracking_service.get_current_status().await {
        Ok(status) => world.current_output = Some(status),
        Err(e) => world.current_error = Some(e.to_string()),
    }
}

// Assertions
#[then("the timer should be running")]
async fn timer_should_be_running(world: &mut TogglWorld) {
    assert!(world.active_timer.is_some());
}

#[then("the timer should be stopped")]
async fn timer_should_be_stopped(world: &mut TogglWorld) {
    assert!(world.active_timer.is_none());
}

#[then(regex = r#"the current project should be "([^"]*)""#)]
async fn current_project_should_be(world: &mut TogglWorld, expected_project: String) {
    if let Some(timer) = &world.active_timer {
        assert_eq!(timer.project_name, expected_project);
    } else {
        panic!("No active timer");
    }
}

#[then(regex = r#"I should get an error "([^"]*)""#)]
async fn should_get_error(world: &mut TogglWorld, expected_error: String) {
    assert!(world.current_error.is_some());
    assert!(world
        .current_error
        .as_ref()
        .unwrap()
        .contains(&expected_error));
}

#[tokio::main]
async fn main() {
    TogglWorld::run("features").await;
}
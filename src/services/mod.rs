pub mod client_discovery;
pub mod git_service;

use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::models::{Project, TimeEntry, Timer, TimeReport};
use crate::repository::Repository;
use crate::{Result, TimeSpanError};

pub use client_discovery::{ClientDiscoveryService, DiscoveryOptions, DiscoveryResult, ClientDirectory};
pub use git_service::GitService;

pub struct ProjectService {
    repository: Arc<dyn Repository>,
}

impl ProjectService {
    pub fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }

    pub async fn create_project(&self, name: &str, description: Option<&str>) -> Result<Project> {
        if let Some(_) = self.repository.get_project_by_name(name).await? {
            return Err(TimeSpanError::ProjectAlreadyExists(name.to_string()));
        }

        let project = Project::new(name.to_string(), description.map(|s| s.to_string()));
        self.repository.create_project(&project).await?;
        Ok(project)
    }

    pub async fn get_project(&self, name: &str) -> Result<Option<Project>> {
        self.repository.get_project_by_name(name).await
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        self.repository.list_projects().await
    }

    pub async fn update_project(&self, name: &str, new_description: Option<String>) -> Result<()> {
        let mut project = self.repository
            .get_project_by_name(name)
            .await?
            .ok_or_else(|| TimeSpanError::ProjectNotFound(name.to_string()))?;

        project.update_description(new_description);
        self.repository.update_project(&project).await
    }

    pub async fn delete_project(&self, name: &str) -> Result<()> {
        let project = self.repository
            .get_project_by_name(name)
            .await?
            .ok_or_else(|| TimeSpanError::ProjectNotFound(name.to_string()))?;

        self.repository.delete_project(project.id).await
    }
}

pub struct TimeTrackingService {
    repository: Arc<dyn Repository>,
}

impl TimeTrackingService {
    pub fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }

    pub async fn start_timer(&self, project_name: &str, task_description: Option<&str>) -> Result<Timer> {
        // Check if there's already an active timer
        if let Some(active) = self.repository.get_active_timer().await? {
            return Err(TimeSpanError::TimerAlreadyRunning(active.project_name));
        }

        // Get project
        let project = self.repository
            .get_project_by_name(project_name)
            .await?
            .ok_or_else(|| TimeSpanError::ProjectNotFound(project_name.to_string()))?;

        let timer = Timer::new(
            project.id,
            project.name,
            task_description.map(|s| s.to_string()),
            Utc::now(),
        );

        // Save the active timer
        self.repository.save_active_timer(&timer).await?;
        
        Ok(timer)
    }

    pub async fn stop_timer(&self) -> Result<TimeEntry> {
        let timer = self.repository
            .get_active_timer()
            .await?
            .ok_or(TimeSpanError::NoActiveTimer)?;

        let end_time = Utc::now();
        
        // Create time entry from timer
        let mut time_entry = TimeEntry::new(
            timer.project_id,
            timer.project_name,
            timer.task_description,
            timer.start_time,
        );
        
        // Set tags from timer
        for tag in timer.tags {
            time_entry.add_tag(tag);
        }
        
        // Stop the entry
        time_entry.stop(end_time)?;
        
        // Save the time entry
        self.repository.create_time_entry(&time_entry).await?;
        
        // Clear the active timer
        self.repository.clear_active_timer().await?;
        
        Ok(time_entry)
    }

    pub async fn get_current_status(&self) -> Result<String> {
        match self.repository.get_active_timer().await? {
            Some(timer) => {
                let elapsed = timer.elapsed();
                let hours = elapsed.num_hours();
                let minutes = elapsed.num_minutes() % 60;
                let task_desc = timer.task_description
                    .map(|desc| format!(" - {}", desc))
                    .unwrap_or_default();
                
                Ok(format!(
                    "⏱️  {} ({}h {}m){}",
                    timer.project_name,
                    hours,
                    minutes,
                    task_desc
                ))
            }
            None => Ok("No active timer".to_string()),
        }
    }

    pub async fn add_tag_to_active_timer(&self, tag: String) -> Result<()> {
        let mut timer = self.repository
            .get_active_timer()
            .await?
            .ok_or(TimeSpanError::NoActiveTimer)?;
            
        timer.add_tag(tag);
        self.repository.save_active_timer(&timer).await
    }
}

pub struct ReportingService {
    repository: Arc<dyn Repository>,
}

impl ReportingService {
    pub fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }

    pub async fn generate_daily_report(&self, date: DateTime<Utc>) -> Result<TimeReport> {
        let start_of_day = date.date_naive().and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Utc).unwrap();
        let end_of_day = date.date_naive().and_hms_opt(23, 59, 59).unwrap().and_local_timezone(Utc).unwrap();
        
        let entries = self.repository
            .list_time_entries_by_date_range(start_of_day, end_of_day)
            .await?;
        
        Ok(TimeReport::new(entries, start_of_day, end_of_day))
    }

    pub async fn generate_weekly_report(&self, date: DateTime<Utc>) -> Result<TimeReport> {
        use chrono::{Datelike, Weekday};
        
        let days_from_monday = match date.weekday() {
            Weekday::Mon => 0,
            Weekday::Tue => 1,
            Weekday::Wed => 2,
            Weekday::Thu => 3,
            Weekday::Fri => 4,
            Weekday::Sat => 5,
            Weekday::Sun => 6,
        };
        
        let monday = date - chrono::Duration::days(days_from_monday as i64);
        let start_of_week = monday.date_naive().and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Utc).unwrap();
        let end_of_week = start_of_week + chrono::Duration::days(6);
        let end_of_week = end_of_week.date_naive().and_hms_opt(23, 59, 59).unwrap().and_local_timezone(Utc).unwrap();
        
        let entries = self.repository
            .list_time_entries_by_date_range(start_of_week, end_of_week)
            .await?;
        
        Ok(TimeReport::new(entries, start_of_week, end_of_week))
    }

    pub async fn generate_project_report(&self, project_name: &str) -> Result<TimeReport> {
        let project = self.repository
            .get_project_by_name(project_name)
            .await?
            .ok_or_else(|| TimeSpanError::ProjectNotFound(project_name.to_string()))?;
        
        let entries = self.repository
            .list_time_entries_by_project(project.id)
            .await?;
        
        // Use earliest and latest entry dates for date range
        let start = entries.iter().map(|e| e.start_time).min().unwrap_or_else(Utc::now);
        let end = entries.iter().filter_map(|e| e.end_time).max().unwrap_or_else(Utc::now);
        
        Ok(TimeReport::new(entries, start, end))
    }
    
    pub fn export_report_json(&self, report: &TimeReport) -> Result<String> {
        serde_json::to_string_pretty(report).map_err(|e| {
            TimeSpanError::InvalidDuration(format!("Failed to serialize report: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::SqliteRepository;
    use chrono::TimeZone;

    async fn setup_services() -> (ProjectService, TimeTrackingService, ReportingService) {
        let repo = Arc::new(SqliteRepository::in_memory().unwrap());
        (
            ProjectService::new(repo.clone()),
            TimeTrackingService::new(repo.clone()),
            ReportingService::new(repo),
        )
    }

    #[tokio::test]
    async fn test_create_project() {
        let (project_service, _, _) = setup_services().await;
        
        let project = project_service
            .create_project("Test Project", Some("Test description"))
            .await
            .unwrap();
        
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.description, Some("Test description".to_string()));
    }

    #[tokio::test]
    async fn test_create_duplicate_project() {
        let (project_service, _, _) = setup_services().await;
        
        project_service
            .create_project("Test Project", None)
            .await
            .unwrap();
        
        let result = project_service
            .create_project("Test Project", None)
            .await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TimeSpanError::ProjectAlreadyExists(_)));
    }

    #[tokio::test]
    async fn test_start_and_stop_timer() {
        let (project_service, tracking_service, _) = setup_services().await;
        
        // Create project
        project_service
            .create_project("Test Project", None)
            .await
            .unwrap();
        
        // Start timer
        let timer = tracking_service
            .start_timer("Test Project", Some("Test task"))
            .await
            .unwrap();
        
        assert_eq!(timer.project_name, "Test Project");
        assert_eq!(timer.task_description, Some("Test task".to_string()));
        
        // Stop timer
        let entry = tracking_service.stop_timer().await.unwrap();
        
        assert_eq!(entry.project_name, "Test Project");
        assert_eq!(entry.task_description, Some("Test task".to_string()));
        assert!(entry.duration.is_some());
        assert!(!entry.is_running());
    }

    #[tokio::test]
    async fn test_start_timer_when_already_running() {
        let (project_service, tracking_service, _) = setup_services().await;
        
        project_service
            .create_project("Project A", None)
            .await
            .unwrap();
        project_service
            .create_project("Project B", None)
            .await
            .unwrap();
        
        tracking_service
            .start_timer("Project A", None)
            .await
            .unwrap();
        
        let result = tracking_service
            .start_timer("Project B", None)
            .await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TimeSpanError::TimerAlreadyRunning(_)));
    }

    #[tokio::test]
    async fn test_stop_timer_when_none_active() {
        let (_, tracking_service, _) = setup_services().await;
        
        let result = tracking_service.stop_timer().await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TimeSpanError::NoActiveTimer));
    }

    #[tokio::test]
    async fn test_get_current_status() {
        let (project_service, tracking_service, _) = setup_services().await;
        
        // No active timer
        let status = tracking_service.get_current_status().await.unwrap();
        assert_eq!(status, "No active timer");
        
        // Start timer
        project_service
            .create_project("Test Project", None)
            .await
            .unwrap();
        
        tracking_service
            .start_timer("Test Project", Some("Test task"))
            .await
            .unwrap();
        
        let status = tracking_service.get_current_status().await.unwrap();
        assert!(status.contains("Test Project"));
        assert!(status.contains("Test task"));
        assert!(status.contains("⏱️"));
    }

    #[tokio::test]
    async fn test_add_tag_to_active_timer() {
        let (project_service, tracking_service, _) = setup_services().await;
        
        project_service
            .create_project("Test Project", None)
            .await
            .unwrap();
        
        tracking_service
            .start_timer("Test Project", None)
            .await
            .unwrap();
        
        tracking_service
            .add_tag_to_active_timer("development".to_string())
            .await
            .unwrap();
        
        let entry = tracking_service.stop_timer().await.unwrap();
        assert_eq!(entry.tags, vec!["development"]);
    }

    #[tokio::test]
    async fn test_export_report_json() {
        let (_, _, reporting_service) = setup_services().await;
        
        let entries = vec![];
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 23, 59, 59).unwrap();
        let report = TimeReport::new(entries, start, end);
        
        let json = reporting_service.export_report_json(&report).unwrap();
        assert!(json.contains("total_duration"));
        assert!(json.contains("entries"));
        assert!(json.contains("project_summaries"));
    }
}

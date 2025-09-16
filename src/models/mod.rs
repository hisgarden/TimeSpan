use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub directory_path: Option<String>,
    pub is_client_project: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: Uuid,
    pub project_id: Uuid,
    pub project_name: String,
    pub task_description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timer {
    pub id: Uuid,
    pub project_id: Uuid,
    pub project_name: String,
    pub task_description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeReport {
    pub total_duration: Duration,
    pub entries: Vec<TimeEntry>,
    pub project_summaries: Vec<ProjectSummary>,
    pub date_range: DateRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub project_name: String,
    pub total_duration: Duration,
    pub entry_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl Project {
    pub fn new(name: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            directory_path: None,
            is_client_project: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn new_client_project(
        name: String,
        description: Option<String>,
        directory_path: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            directory_path: Some(directory_path),
            is_client_project: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }
}

impl TimeEntry {
    pub fn new(
        project_id: Uuid,
        project_name: String,
        task_description: Option<String>,
        start_time: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            project_id,
            project_name,
            task_description,
            start_time,
            end_time: None,
            duration: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn stop(&mut self, end_time: DateTime<Utc>) -> crate::Result<()> {
        if end_time <= self.start_time {
            return Err(crate::TimeSpanError::InvalidDuration(
                "End time must be after start time".to_string(),
            ));
        }

        self.end_time = Some(end_time);
        self.duration = Some(end_time - self.start_time);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
        self.updated_at = Utc::now();
    }

    pub fn is_running(&self) -> bool {
        self.end_time.is_none()
    }

    pub fn current_duration(&self) -> Duration {
        match self.end_time {
            Some(end) => end - self.start_time,
            None => Utc::now() - self.start_time,
        }
    }
}

impl Timer {
    pub fn new(
        project_id: Uuid,
        project_name: String,
        task_description: Option<String>,
        start_time: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id,
            project_name,
            task_description,
            start_time,
            tags: Vec::new(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        Utc::now() - self.start_time
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

impl TimeReport {
    pub fn new(entries: Vec<TimeEntry>, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        let total_duration = entries
            .iter()
            .filter_map(|e| e.duration)
            .fold(Duration::zero(), |acc, d| acc + d);

        let mut project_summaries = std::collections::HashMap::new();

        for entry in &entries {
            let summary = project_summaries
                .entry(entry.project_name.clone())
                .or_insert_with(|| ProjectSummary {
                    project_name: entry.project_name.clone(),
                    total_duration: Duration::zero(),
                    entry_count: 0,
                });

            if let Some(duration) = entry.duration {
                summary.total_duration += duration;
            }
            summary.entry_count += 1;
        }

        let project_summaries: Vec<ProjectSummary> = project_summaries.into_values().collect();

        Self {
            total_duration,
            entries,
            project_summaries,
            date_range: DateRange { start, end },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_project_creation() {
        let project = Project::new("Test Project".to_string(), None);

        assert_eq!(project.name, "Test Project");
        assert!(project.description.is_none());
        assert!(project.created_at <= Utc::now());
        assert_eq!(project.created_at, project.updated_at);
    }

    #[test]
    fn test_project_with_description() {
        let description = Some("A test project description".to_string());
        let project = Project::new("Test Project".to_string(), description.clone());

        assert_eq!(project.description, description);
    }

    #[test]
    fn test_project_update_description() {
        let mut project = Project::new("Test Project".to_string(), None);
        let original_updated_at = project.updated_at;

        // Small delay to ensure updated_at changes
        std::thread::sleep(std::time::Duration::from_millis(1));

        project.update_description(Some("New description".to_string()));

        assert_eq!(project.description, Some("New description".to_string()));
        assert!(project.updated_at > original_updated_at);
    }

    #[test]
    fn test_time_entry_creation() {
        let project_id = Uuid::new_v4();
        let start_time = Utc::now();

        let entry = TimeEntry::new(
            project_id,
            "Test Project".to_string(),
            Some("Test task".to_string()),
            start_time,
        );

        assert_eq!(entry.project_id, project_id);
        assert_eq!(entry.project_name, "Test Project");
        assert_eq!(entry.task_description, Some("Test task".to_string()));
        assert_eq!(entry.start_time, start_time);
        assert!(entry.end_time.is_none());
        assert!(entry.duration.is_none());
        assert!(entry.is_running());
    }

    #[test]
    fn test_time_entry_stop() {
        let project_id = Uuid::new_v4();
        let start_time = Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap();
        let end_time = Utc.with_ymd_and_hms(2024, 1, 1, 10, 30, 0).unwrap();

        let mut entry = TimeEntry::new(project_id, "Test Project".to_string(), None, start_time);

        let result = entry.stop(end_time);

        assert!(result.is_ok());
        assert_eq!(entry.end_time, Some(end_time));
        assert_eq!(entry.duration, Some(Duration::minutes(90)));
        assert!(!entry.is_running());
    }

    #[test]
    fn test_time_entry_stop_invalid_end_time() {
        let project_id = Uuid::new_v4();
        let start_time = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
        let end_time = Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap(); // Before start time

        let mut entry = TimeEntry::new(project_id, "Test Project".to_string(), None, start_time);

        let result = entry.stop(end_time);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::TimeSpanError::InvalidDuration(_)
        ));
    }

    #[test]
    fn test_time_entry_tags() {
        let project_id = Uuid::new_v4();
        let mut entry = TimeEntry::new(project_id, "Test Project".to_string(), None, Utc::now());

        entry.add_tag("development".to_string());
        entry.add_tag("rust".to_string());
        entry.add_tag("development".to_string()); // Duplicate should be ignored

        assert_eq!(entry.tags, vec!["development", "rust"]);

        entry.remove_tag("rust");
        assert_eq!(entry.tags, vec!["development"]);
    }

    #[test]
    fn test_time_entry_current_duration() {
        let project_id = Uuid::new_v4();
        let start_time = Utc::now() - Duration::minutes(30);

        let entry = TimeEntry::new(project_id, "Test Project".to_string(), None, start_time);

        let current_duration = entry.current_duration();
        assert!(current_duration >= Duration::minutes(29)); // Allow for test execution time
        assert!(current_duration <= Duration::minutes(31));
    }

    #[test]
    fn test_timer_creation() {
        let project_id = Uuid::new_v4();
        let start_time = Utc::now();

        let timer = Timer::new(
            project_id,
            "Test Project".to_string(),
            Some("Test task".to_string()),
            start_time,
        );

        assert_eq!(timer.project_id, project_id);
        assert_eq!(timer.project_name, "Test Project");
        assert_eq!(timer.task_description, Some("Test task".to_string()));
        assert_eq!(timer.start_time, start_time);
    }

    #[test]
    fn test_timer_elapsed() {
        let project_id = Uuid::new_v4();
        let start_time = Utc::now() - Duration::minutes(15);

        let timer = Timer::new(project_id, "Test Project".to_string(), None, start_time);

        let elapsed = timer.elapsed();
        assert!(elapsed >= Duration::minutes(14)); // Allow for test execution time
        assert!(elapsed <= Duration::minutes(16));
    }

    #[test]
    fn test_timer_tags() {
        let project_id = Uuid::new_v4();
        let mut timer = Timer::new(project_id, "Test Project".to_string(), None, Utc::now());

        timer.add_tag("development".to_string());
        timer.add_tag("rust".to_string());
        timer.add_tag("development".to_string()); // Duplicate should be ignored

        assert_eq!(timer.tags, vec!["development", "rust"]);
    }

    #[test]
    fn test_time_report_creation() {
        let project_id = Uuid::new_v4();
        let start_time = Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap();
        let end_time = Utc.with_ymd_and_hms(2024, 1, 1, 17, 0, 0).unwrap();

        let mut entry1 = TimeEntry::new(
            project_id,
            "Project A".to_string(),
            None,
            Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap(),
        );
        entry1
            .stop(Utc.with_ymd_and_hms(2024, 1, 1, 11, 0, 0).unwrap())
            .unwrap();

        let mut entry2 = TimeEntry::new(
            project_id,
            "Project B".to_string(),
            None,
            Utc.with_ymd_and_hms(2024, 1, 1, 13, 0, 0).unwrap(),
        );
        entry2
            .stop(Utc.with_ymd_and_hms(2024, 1, 1, 15, 30, 0).unwrap())
            .unwrap();

        let entries = vec![entry1, entry2];
        let report = TimeReport::new(entries, start_time, end_time);

        assert_eq!(report.total_duration, Duration::minutes(270)); // 2h + 2.5h = 4.5h
        assert_eq!(report.entries.len(), 2);
        assert_eq!(report.project_summaries.len(), 2);
        assert_eq!(report.date_range.start, start_time);
        assert_eq!(report.date_range.end, end_time);
    }
}

// Git Integration Models
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub author_email: String,
    pub timestamp: DateTime<Utc>,
    pub files_changed: Vec<String>,
    pub insertions: u32,
    pub deletions: u32,
    pub repository_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitTimeEntry {
    pub id: Uuid,
    pub commit_hash: String,
    pub project_id: Uuid,
    pub project_name: String,
    pub estimated_time: Duration,
    pub actual_time: Option<Duration>,
    pub confidence_score: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CommitAnalysis {
    pub commit: GitCommit,
    pub complexity_score: f32,
    pub file_type_weights: std::collections::HashMap<String, f32>,
    pub commit_type: CommitType,
    pub estimated_duration: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommitType {
    Feature,
    BugFix,
    Refactor,
    Documentation,
    Test,
    Chore,
    Other,
}

impl GitCommit {
    pub fn new(
        hash: String,
        message: String,
        author: String,
        author_email: String,
        timestamp: DateTime<Utc>,
        repository_path: PathBuf,
    ) -> Self {
        Self {
            hash,
            message,
            author,
            author_email,
            timestamp,
            files_changed: Vec::new(),
            insertions: 0,
            deletions: 0,
            repository_path,
        }
    }

    pub fn total_changes(&self) -> u32 {
        self.insertions + self.deletions
    }

    pub fn detect_commit_type(&self) -> CommitType {
        let msg = self.message.to_lowercase();

        if msg.starts_with("feat") || msg.contains("feature") || msg.contains("add") {
            CommitType::Feature
        } else if msg.starts_with("fix") || msg.contains("bug") || msg.contains("error") {
            CommitType::BugFix
        } else if msg.starts_with("refactor") || msg.contains("refactor") {
            CommitType::Refactor
        } else if msg.starts_with("docs") || msg.contains("documentation") {
            CommitType::Documentation
        } else if msg.starts_with("test") || msg.contains("test") {
            CommitType::Test
        } else if msg.starts_with("chore") || msg.contains("chore") {
            CommitType::Chore
        } else {
            CommitType::Other
        }
    }
}

impl GitTimeEntry {
    pub fn new(
        commit_hash: String,
        project_id: Uuid,
        project_name: String,
        estimated_time: Duration,
        confidence_score: f32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            commit_hash,
            project_id,
            project_name,
            estimated_time,
            actual_time: None,
            confidence_score,
            created_at: Utc::now(),
        }
    }

    pub fn set_actual_time(&mut self, actual_time: Duration) {
        self.actual_time = Some(actual_time);
    }
}

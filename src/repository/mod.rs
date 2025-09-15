use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use std::path::Path;
use uuid::Uuid;

use crate::models::{Project, TimeEntry, Timer};
use crate::{Result, TimeSpanError};

#[async_trait]
pub trait Repository: Send + Sync {
    async fn create_project(&self, project: &Project) -> Result<()>;
    async fn get_project_by_name(&self, name: &str) -> Result<Option<Project>>;
    async fn get_project_by_id(&self, id: Uuid) -> Result<Option<Project>>;
    async fn list_projects(&self) -> Result<Vec<Project>>;
    async fn update_project(&self, project: &Project) -> Result<()>;
    async fn delete_project(&self, id: Uuid) -> Result<()>;

    async fn create_time_entry(&self, entry: &TimeEntry) -> Result<()>;
    async fn get_time_entry_by_id(&self, id: Uuid) -> Result<Option<TimeEntry>>;
    async fn get_active_time_entry(&self) -> Result<Option<TimeEntry>>;
    async fn list_time_entries_by_project(&self, project_id: Uuid) -> Result<Vec<TimeEntry>>;
    async fn list_time_entries_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<TimeEntry>>;
    async fn update_time_entry(&self, entry: &TimeEntry) -> Result<()>;
    async fn count_time_entries_for_project(&self, project_id: Uuid) -> Result<usize>;

    async fn save_active_timer(&self, timer: &Timer) -> Result<()>;
    async fn get_active_timer(&self) -> Result<Option<Timer>>;
    async fn clear_active_timer(&self) -> Result<()>;

    // Test helper methods
    async fn clear_all(&self) -> Result<()>;
}

pub struct SqliteRepository {
    connection: std::sync::Mutex<Connection>,
}

impl SqliteRepository {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let repo = Self {
            connection: std::sync::Mutex::new(conn),
        };
        repo.create_tables()?;
        Ok(repo)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let repo = Self {
            connection: std::sync::Mutex::new(conn),
        };
        repo.create_tables()?;
        Ok(repo)
    }

    fn create_tables(&self) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
            [],
        )?;

        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS time_entries (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                project_name TEXT NOT NULL,
                task_description TEXT,
                start_time TEXT NOT NULL,
                end_time TEXT,
                duration_seconds INTEGER,
                tags TEXT, -- JSON array
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects (id)
            )
            "#,
            [],
        )?;

        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS active_timer (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                project_name TEXT NOT NULL,
                task_description TEXT,
                start_time TEXT NOT NULL,
                tags TEXT -- JSON array
            )
            "#,
            [],
        )?;

        Ok(())
    }

    fn project_from_row(row: &Row) -> rusqlite::Result<Project> {
        Ok(Project {
            id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
            name: row.get("name")?,
            description: row.get("description")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .unwrap()
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("updated_at")?)
                .unwrap()
                .with_timezone(&Utc),
        })
    }

    fn time_entry_from_row(row: &Row) -> rusqlite::Result<TimeEntry> {
        let tags_json: Option<String> = row.get("tags")?;
        let tags = if let Some(json) = tags_json {
            serde_json::from_str(&json).unwrap_or_default()
        } else {
            Vec::new()
        };

        let end_time_str: Option<String> = row.get("end_time")?;
        let end_time = end_time_str.map(|s| {
            DateTime::parse_from_rfc3339(&s)
                .unwrap()
                .with_timezone(&Utc)
        });

        let duration_seconds: Option<i64> = row.get("duration_seconds")?;
        let duration = duration_seconds.map(|s| chrono::Duration::seconds(s));

        Ok(TimeEntry {
            id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
            project_id: Uuid::parse_str(&row.get::<_, String>("project_id")?).unwrap(),
            project_name: row.get("project_name")?,
            task_description: row.get("task_description")?,
            start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>("start_time")?)
                .unwrap()
                .with_timezone(&Utc),
            end_time,
            duration,
            tags,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .unwrap()
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("updated_at")?)
                .unwrap()
                .with_timezone(&Utc),
        })
    }

    fn timer_from_row(row: &Row) -> rusqlite::Result<Timer> {
        let tags_json: Option<String> = row.get("tags")?;
        let tags = if let Some(json) = tags_json {
            serde_json::from_str(&json).unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(Timer {
            id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
            project_id: Uuid::parse_str(&row.get::<_, String>("project_id")?).unwrap(),
            project_name: row.get("project_name")?,
            task_description: row.get("task_description")?,
            start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>("start_time")?)
                .unwrap()
                .with_timezone(&Utc),
            tags,
        })
    }
}

#[async_trait]
impl Repository for SqliteRepository {
    async fn create_project(&self, project: &Project) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        
        let result = conn.execute(
            "INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                project.id.to_string(),
                project.name,
                project.description,
                project.created_at.to_rfc3339(),
                project.updated_at.to_rfc3339(),
            ],
        );

        match result {
            Ok(_) => Ok(()),
            Err(rusqlite::Error::SqliteFailure(err, _)) => {
                if err.code == rusqlite::ErrorCode::ConstraintViolation {
                    Err(TimeSpanError::ProjectAlreadyExists(project.name.clone()))
                } else {
                    Err(TimeSpanError::Database(rusqlite::Error::SqliteFailure(err, None)))
                }
            }
            Err(e) => Err(TimeSpanError::Database(e)),
        }
    }

    async fn get_project_by_name(&self, name: &str) -> Result<Option<Project>> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare("SELECT id, name, description, created_at, updated_at FROM projects WHERE name = ?1")?;
        let mut rows = stmt.query_map(params![name], Self::project_from_row)?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    async fn get_project_by_id(&self, id: Uuid) -> Result<Option<Project>> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare("SELECT id, name, description, created_at, updated_at FROM projects WHERE id = ?1")?;
        let mut rows = stmt.query_map(params![id.to_string()], Self::project_from_row)?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    async fn list_projects(&self) -> Result<Vec<Project>> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare("SELECT id, name, description, created_at, updated_at FROM projects ORDER BY name")?;
        let project_iter = stmt.query_map([], Self::project_from_row)?;
        
        let mut projects = Vec::new();
        for project in project_iter {
            projects.push(project?);
        }
        
        Ok(projects)
    }

    async fn update_project(&self, project: &Project) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        
        conn.execute(
            "UPDATE projects SET name = ?2, description = ?3, updated_at = ?4 WHERE id = ?1",
            params![
                project.id.to_string(),
                project.name,
                project.description,
                project.updated_at.to_rfc3339(),
            ],
        )?;
        
        Ok(())
    }

    async fn delete_project(&self, id: Uuid) -> Result<()> {
        // Check if project has time entries (need to do this outside the lock)
        let project = self.get_project_by_id(id).await?;
        let project_name = project.as_ref().map(|p| p.name.clone()).unwrap_or_else(|| id.to_string());
        
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM time_entries WHERE project_id = ?1")?;
        let count: i64 = stmt.query_row(params![id.to_string()], |row| row.get(0))?;
        
        if count > 0 {
            return Err(TimeSpanError::ProjectHasTimeEntries(project_name));
        }
        
        conn.execute("DELETE FROM projects WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    async fn create_time_entry(&self, entry: &TimeEntry) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        
        let tags_json = if entry.tags.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&entry.tags).unwrap())
        };

        let duration_seconds = entry.duration.map(|d| d.num_seconds());
        let end_time = entry.end_time.map(|dt| dt.to_rfc3339());
        
        conn.execute(
            r#"
            INSERT INTO time_entries 
            (id, project_id, project_name, task_description, start_time, end_time, duration_seconds, tags, created_at, updated_at) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                entry.id.to_string(),
                entry.project_id.to_string(),
                entry.project_name,
                entry.task_description,
                entry.start_time.to_rfc3339(),
                end_time,
                duration_seconds,
                tags_json,
                entry.created_at.to_rfc3339(),
                entry.updated_at.to_rfc3339(),
            ],
        )?;
        
        Ok(())
    }

    async fn get_time_entry_by_id(&self, id: Uuid) -> Result<Option<TimeEntry>> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, project_id, project_name, task_description, start_time, end_time, 
                   duration_seconds, tags, created_at, updated_at 
            FROM time_entries WHERE id = ?1
            "#
        )?;
        let mut rows = stmt.query_map(params![id.to_string()], Self::time_entry_from_row)?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    async fn get_active_time_entry(&self) -> Result<Option<TimeEntry>> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, project_id, project_name, task_description, start_time, end_time, 
                   duration_seconds, tags, created_at, updated_at 
            FROM time_entries WHERE end_time IS NULL
            ORDER BY start_time DESC LIMIT 1
            "#
        )?;
        let mut rows = stmt.query_map([], Self::time_entry_from_row)?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    async fn list_time_entries_by_project(&self, project_id: Uuid) -> Result<Vec<TimeEntry>> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, project_id, project_name, task_description, start_time, end_time, 
                   duration_seconds, tags, created_at, updated_at 
            FROM time_entries WHERE project_id = ?1
            ORDER BY start_time DESC
            "#
        )?;
        let entry_iter = stmt.query_map(params![project_id.to_string()], Self::time_entry_from_row)?;
        
        let mut entries = Vec::new();
        for entry in entry_iter {
            entries.push(entry?);
        }
        
        Ok(entries)
    }

    async fn list_time_entries_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<TimeEntry>> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, project_id, project_name, task_description, start_time, end_time, 
                   duration_seconds, tags, created_at, updated_at 
            FROM time_entries 
            WHERE start_time >= ?1 AND start_time <= ?2
            ORDER BY start_time ASC
            "#
        )?;
        let entry_iter = stmt.query_map(
            params![start.to_rfc3339(), end.to_rfc3339()], 
            Self::time_entry_from_row
        )?;
        
        let mut entries = Vec::new();
        for entry in entry_iter {
            entries.push(entry?);
        }
        
        Ok(entries)
    }

    async fn update_time_entry(&self, entry: &TimeEntry) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        
        let tags_json = if entry.tags.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&entry.tags).unwrap())
        };

        let duration_seconds = entry.duration.map(|d| d.num_seconds());
        let end_time = entry.end_time.map(|dt| dt.to_rfc3339());
        
        conn.execute(
            r#"
            UPDATE time_entries 
            SET project_id = ?2, project_name = ?3, task_description = ?4, start_time = ?5, 
                end_time = ?6, duration_seconds = ?7, tags = ?8, updated_at = ?9
            WHERE id = ?1
            "#,
            params![
                entry.id.to_string(),
                entry.project_id.to_string(),
                entry.project_name,
                entry.task_description,
                entry.start_time.to_rfc3339(),
                end_time,
                duration_seconds,
                tags_json,
                entry.updated_at.to_rfc3339(),
            ],
        )?;
        
        Ok(())
    }

    async fn count_time_entries_for_project(&self, project_id: Uuid) -> Result<usize> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM time_entries WHERE project_id = ?1")?;
        let count: i64 = stmt.query_row(params![project_id.to_string()], |row| row.get(0))?;
        
        Ok(count as usize)
    }

    async fn save_active_timer(&self, timer: &Timer) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        
        // Clear any existing active timer
        conn.execute("DELETE FROM active_timer", [])?;
        
        let tags_json = if timer.tags.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&timer.tags).unwrap())
        };
        
        conn.execute(
            r#"
            INSERT INTO active_timer 
            (id, project_id, project_name, task_description, start_time, tags) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                timer.id.to_string(),
                timer.project_id.to_string(),
                timer.project_name,
                timer.task_description,
                timer.start_time.to_rfc3339(),
                tags_json,
            ],
        )?;
        
        Ok(())
    }

    async fn get_active_timer(&self) -> Result<Option<Timer>> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, project_id, project_name, task_description, start_time, tags FROM active_timer"
        )?;
        let mut rows = stmt.query_map([], Self::timer_from_row)?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    async fn clear_active_timer(&self) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute("DELETE FROM active_timer", [])?;
        Ok(())
    }

    async fn clear_all(&self) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute("DELETE FROM time_entries", [])?;
        conn.execute("DELETE FROM projects", [])?;
        conn.execute("DELETE FROM active_timer", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    async fn setup_repo() -> SqliteRepository {
        SqliteRepository::in_memory().unwrap()
    }

    #[tokio::test]
    async fn test_create_and_get_project() {
        let repo = setup_repo().await;
        let project = Project::new("Test Project".to_string(), Some("Description".to_string()));
        
        repo.create_project(&project).await.unwrap();
        
        let retrieved = repo.get_project_by_name("Test Project").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "Test Project");
        assert_eq!(retrieved.description, Some("Description".to_string()));
    }

    #[tokio::test]
    async fn test_create_duplicate_project_fails() {
        let repo = setup_repo().await;
        let project1 = Project::new("Test Project".to_string(), None);
        let project2 = Project::new("Test Project".to_string(), None);
        
        repo.create_project(&project1).await.unwrap();
        
        let result = repo.create_project(&project2).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TimeSpanError::ProjectAlreadyExists(_)));
    }

    #[tokio::test]
    async fn test_list_projects() {
        let repo = setup_repo().await;
        let project1 = Project::new("Project A".to_string(), None);
        let project2 = Project::new("Project B".to_string(), None);
        
        repo.create_project(&project1).await.unwrap();
        repo.create_project(&project2).await.unwrap();
        
        let projects = repo.list_projects().await.unwrap();
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].name, "Project A"); // Should be sorted alphabetically
        assert_eq!(projects[1].name, "Project B");
    }

    #[tokio::test]
    async fn test_update_project() {
        let repo = setup_repo().await;
        let mut project = Project::new("Test Project".to_string(), None);
        
        repo.create_project(&project).await.unwrap();
        
        project.update_description(Some("New description".to_string()));
        repo.update_project(&project).await.unwrap();
        
        let retrieved = repo.get_project_by_id(project.id).await.unwrap().unwrap();
        assert_eq!(retrieved.description, Some("New description".to_string()));
        assert_eq!(retrieved.updated_at, project.updated_at);
    }

    #[tokio::test]
    async fn test_delete_project_without_entries() {
        let repo = setup_repo().await;
        let project = Project::new("Test Project".to_string(), None);
        
        repo.create_project(&project).await.unwrap();
        repo.delete_project(project.id).await.unwrap();
        
        let retrieved = repo.get_project_by_id(project.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_delete_project_with_entries_fails() {
        let repo = setup_repo().await;
        let project = Project::new("Test Project".to_string(), None);
        repo.create_project(&project).await.unwrap();
        
        let entry = TimeEntry::new(
            project.id,
            project.name.clone(),
            None,
            Utc::now(),
        );
        repo.create_time_entry(&entry).await.unwrap();
        
        let result = repo.delete_project(project.id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TimeSpanError::ProjectHasTimeEntries(_)));
    }

    #[tokio::test]
    async fn test_create_and_get_time_entry() {
        let repo = setup_repo().await;
        let project = Project::new("Test Project".to_string(), None);
        repo.create_project(&project).await.unwrap();
        
        let mut entry = TimeEntry::new(
            project.id,
            project.name.clone(),
            Some("Test task".to_string()),
            Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap(),
        );
        entry.add_tag("development".to_string());
        
        repo.create_time_entry(&entry).await.unwrap();
        
        let retrieved = repo.get_time_entry_by_id(entry.id).await.unwrap().unwrap();
        assert_eq!(retrieved.project_name, "Test Project");
        assert_eq!(retrieved.task_description, Some("Test task".to_string()));
        assert_eq!(retrieved.tags, vec!["development"]);
        assert!(retrieved.is_running());
    }

    #[tokio::test]
    async fn test_get_active_time_entry() {
        let repo = setup_repo().await;
        let project = Project::new("Test Project".to_string(), None);
        repo.create_project(&project).await.unwrap();
        
        // Create a finished entry
        let mut finished_entry = TimeEntry::new(
            project.id,
            project.name.clone(),
            None,
            Utc.with_ymd_and_hms(2024, 1, 1, 8, 0, 0).unwrap(),
        );
        finished_entry.stop(Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap()).unwrap();
        repo.create_time_entry(&finished_entry).await.unwrap();
        
        // Create an active entry
        let active_entry = TimeEntry::new(
            project.id,
            project.name.clone(),
            None,
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
        );
        repo.create_time_entry(&active_entry).await.unwrap();
        
        let retrieved = repo.get_active_time_entry().await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, active_entry.id);
    }

    #[tokio::test]
    async fn test_update_time_entry() {
        let repo = setup_repo().await;
        let project = Project::new("Test Project".to_string(), None);
        repo.create_project(&project).await.unwrap();
        
        let mut entry = TimeEntry::new(
            project.id,
            project.name.clone(),
            None,
            Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap(),
        );
        repo.create_time_entry(&entry).await.unwrap();
        
        entry.stop(Utc.with_ymd_and_hms(2024, 1, 1, 10, 30, 0).unwrap()).unwrap();
        repo.update_time_entry(&entry).await.unwrap();
        
        let retrieved = repo.get_time_entry_by_id(entry.id).await.unwrap().unwrap();
        assert!(!retrieved.is_running());
        assert_eq!(retrieved.duration, Some(chrono::Duration::minutes(90)));
    }

    #[tokio::test]
    async fn test_list_time_entries_by_project() {
        let repo = setup_repo().await;
        let project1 = Project::new("Project 1".to_string(), None);
        let project2 = Project::new("Project 2".to_string(), None);
        repo.create_project(&project1).await.unwrap();
        repo.create_project(&project2).await.unwrap();
        
        let entry1 = TimeEntry::new(project1.id, project1.name.clone(), None, Utc::now());
        let entry2 = TimeEntry::new(project1.id, project1.name.clone(), None, Utc::now());
        let entry3 = TimeEntry::new(project2.id, project2.name.clone(), None, Utc::now());
        
        repo.create_time_entry(&entry1).await.unwrap();
        repo.create_time_entry(&entry2).await.unwrap();
        repo.create_time_entry(&entry3).await.unwrap();
        
        let project1_entries = repo.list_time_entries_by_project(project1.id).await.unwrap();
        assert_eq!(project1_entries.len(), 2);
        
        let project2_entries = repo.list_time_entries_by_project(project2.id).await.unwrap();
        assert_eq!(project2_entries.len(), 1);
    }

    #[tokio::test]
    async fn test_active_timer_operations() {
        let repo = setup_repo().await;
        let project = Project::new("Test Project".to_string(), None);
        repo.create_project(&project).await.unwrap();
        
        let mut timer = Timer::new(
            project.id,
            project.name.clone(),
            Some("Test task".to_string()),
            Utc::now(),
        );
        timer.add_tag("development".to_string());
        
        // No active timer initially
        let active = repo.get_active_timer().await.unwrap();
        assert!(active.is_none());
        
        // Save timer
        repo.save_active_timer(&timer).await.unwrap();
        
        let active = repo.get_active_timer().await.unwrap();
        assert!(active.is_some());
        let active = active.unwrap();
        assert_eq!(active.project_name, "Test Project");
        assert_eq!(active.task_description, Some("Test task".to_string()));
        assert_eq!(active.tags, vec!["development"]);
        
        // Clear timer
        repo.clear_active_timer().await.unwrap();
        
        let active = repo.get_active_timer().await.unwrap();
        assert!(active.is_none());
    }

    #[tokio::test]
    async fn test_count_time_entries_for_project() {
        let repo = setup_repo().await;
        let project = Project::new("Test Project".to_string(), None);
        repo.create_project(&project).await.unwrap();
        
        assert_eq!(repo.count_time_entries_for_project(project.id).await.unwrap(), 0);
        
        let entry1 = TimeEntry::new(project.id, project.name.clone(), None, Utc::now());
        let entry2 = TimeEntry::new(project.id, project.name.clone(), None, Utc::now());
        repo.create_time_entry(&entry1).await.unwrap();
        repo.create_time_entry(&entry2).await.unwrap();
        
        assert_eq!(repo.count_time_entries_for_project(project.id).await.unwrap(), 2);
    }
}
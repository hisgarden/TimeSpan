use std::path::Path;
use std::collections::HashMap;
use chrono::{DateTime, Duration, TimeZone, Utc};
use git2::{Repository, Commit};

use crate::models::{GitCommit, GitTimeEntry, CommitAnalysis, CommitType, Project};
use crate::repository::Repository as TimeSpanRepository;
use crate::{Result, TimeSpanError};

pub struct GitService {
    repository: std::sync::Arc<dyn TimeSpanRepository>,
}

impl GitService {
    pub fn new(repository: std::sync::Arc<dyn TimeSpanRepository>) -> Self {
        Self { repository }
    }

    /// Get commits from a local git repository
    pub async fn get_commits(
        &self,
        repo_path: &Path,
        since: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Result<Vec<GitCommit>> {
        let git_repo = Repository::open(repo_path)
            .map_err(|e| TimeSpanError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, 
                format!("Failed to open git repository at {}: {}", repo_path.display(), e))))?;

        let mut revwalk = git_repo.revwalk()
            .map_err(|e| TimeSpanError::Io(std::io::Error::new(std::io::ErrorKind::Other, 
                format!("Failed to create revwalk: {}", e))))?;

        revwalk.push_head()
            .map_err(|e| TimeSpanError::Io(std::io::Error::new(std::io::ErrorKind::Other, 
                format!("Failed to push HEAD: {}", e))))?;

        let mut commits = Vec::new();
        let mut count = 0;

        for oid in revwalk {
            if let Some(max_count) = limit {
                if count >= max_count {
                    break;
                }
            }

            let oid = oid.map_err(|e| TimeSpanError::Io(std::io::Error::new(std::io::ErrorKind::Other, 
                format!("Failed to get commit OID: {}", e))))?;

            let commit_obj = git_repo.find_commit(oid)
                .map_err(|e| TimeSpanError::Io(std::io::Error::new(std::io::ErrorKind::Other, 
                    format!("Failed to find commit: {}", e))))?;

            let commit_time = Utc.timestamp_opt(commit_obj.time().seconds(), 0)
                .single()
                .ok_or_else(|| TimeSpanError::InvalidDuration("Invalid commit timestamp".to_string()))?;

            // Filter by date if specified
            if let Some(since_date) = since {
                if commit_time < since_date {
                    break;
                }
            }

            let mut git_commit = GitCommit::new(
                oid.to_string(),
                commit_obj.message().unwrap_or("").to_string(),
                commit_obj.author().name().unwrap_or("Unknown").to_string(),
                commit_obj.author().email().unwrap_or("").to_string(),
                commit_time,
                repo_path.to_path_buf(),
            );

            // Get file changes for this commit
            if let Ok((files, insertions, deletions)) = self.get_commit_stats(&git_repo, &commit_obj) {
                git_commit.files_changed = files;
                git_commit.insertions = insertions;
                git_commit.deletions = deletions;
            }

            commits.push(git_commit);
            count += 1;
        }

        Ok(commits)
    }

    /// Get statistics for a specific commit (files changed, insertions, deletions)
    fn get_commit_stats(&self, repo: &Repository, commit: &Commit) -> std::result::Result<(Vec<String>, u32, u32), git2::Error> {
        let mut files_changed = Vec::new();
        let mut total_insertions = 0u32;
        let mut total_deletions = 0u32;

        let tree = commit.tree()?;
        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };

        let mut diff_options = git2::DiffOptions::new();
        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut diff_options))?;

        diff.foreach(
            &mut |delta, _progress| {
                if let Some(path) = delta.new_file().path() {
                    files_changed.push(path.to_string_lossy().to_string());
                }
                true
            },
            None,
            None,
            Some(&mut |_delta, _hunk, line| {
                match line.origin() {
                    '+' => total_insertions += 1,
                    '-' => total_deletions += 1,
                    _ => {}
                }
                true
            }),
        )?;

        Ok((files_changed, total_insertions, total_deletions))
    }

    /// Analyze a commit and estimate time spent
    pub async fn analyze_commit(&self, commit: &GitCommit) -> Result<CommitAnalysis> {
        let commit_type = commit.detect_commit_type();
        let complexity_score = self.calculate_complexity_score(commit);
        let file_type_weights = self.get_file_type_weights(&commit.files_changed);
        let estimated_duration = self.estimate_commit_time(commit, &commit_type, complexity_score);

        Ok(CommitAnalysis {
            commit: commit.clone(),
            complexity_score,
            file_type_weights,
            commit_type,
            estimated_duration,
        })
    }

    /// Calculate complexity score based on changes
    fn calculate_complexity_score(&self, commit: &GitCommit) -> f32 {
        let total_changes = commit.total_changes() as f32;
        let file_count = commit.files_changed.len() as f32;
        
        // Base complexity on lines changed and files touched
        let lines_score = (total_changes / 100.0).min(3.0);  // Cap at 3.0 for very large commits
        let files_score = (file_count / 10.0).min(2.0);      // Cap at 2.0 for many files
        
        (lines_score + files_score) / 2.0
    }

    /// Get weights for different file types
    fn get_file_type_weights(&self, files: &[String]) -> HashMap<String, f32> {
        let mut weights = HashMap::new();
        
        for file in files {
            let weight = match Path::new(file).extension().and_then(|ext| ext.to_str()) {
                Some("rs") => 1.5,      // Rust files are complex
                Some("py") => 1.3,      // Python files
                Some("js") | Some("ts") => 1.2,  // JavaScript/TypeScript
                Some("java") | Some("cpp") | Some("c") => 1.4,  // Compiled languages
                Some("md") | Some("txt") => 0.5,  // Documentation
                Some("json") | Some("toml") | Some("yaml") | Some("yml") => 0.3,  // Config files
                Some("html") | Some("css") => 0.7,  // Frontend markup/styling
                _ => 1.0,  // Default weight
            };
            
            let ext = Path::new(file)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("unknown")
                .to_string();
            
            *weights.entry(ext).or_insert(0.0) += weight;
        }
        
        weights
    }

    /// Estimate time spent on a commit
    fn estimate_commit_time(&self, commit: &GitCommit, commit_type: &CommitType, complexity_score: f32) -> Duration {
        let mut base_time = match commit_type {
            CommitType::Feature => Duration::minutes(45),      // Features take longer
            CommitType::BugFix => Duration::minutes(60),       // Bug fixes can be tricky
            CommitType::Refactor => Duration::minutes(30),     // Refactoring
            CommitType::Documentation => Duration::minutes(15), // Documentation is usually quick
            CommitType::Test => Duration::minutes(25),         // Writing tests
            CommitType::Chore => Duration::minutes(10),        // Chores are usually quick
            CommitType::Other => Duration::minutes(20),        // Default
        };

        // Adjust based on complexity
        let complexity_multiplier = 1.0 + (complexity_score * 0.5);
        let adjusted_minutes = (base_time.num_minutes() as f32 * complexity_multiplier) as i64;
        base_time = Duration::minutes(adjusted_minutes);

        // Adjust based on total changes
        let changes_factor = (commit.total_changes() as f32 / 50.0).min(3.0);  // Cap the multiplier
        base_time = base_time + Duration::minutes((changes_factor * 10.0) as i64);

        // Apply file type weights
        let file_weight_bonus = commit.files_changed.iter()
            .map(|file| {
                match Path::new(file).extension().and_then(|ext| ext.to_str()) {
                    Some("rs") => 5,      // Extra time for Rust
                    Some("js") | Some("ts") => 3,
                    Some("md") => 1,
                    _ => 0,
                }
            })
            .sum::<i64>();

        base_time = base_time + Duration::minutes(file_weight_bonus);

        // Cap the maximum time per commit
        base_time.min(Duration::hours(4))
    }

    /// Detect project from repository path
    pub async fn detect_project(&self, repo_path: &Path) -> Result<Option<String>> {
        // Try to find project based on directory name or remote URL
        let dir_name = repo_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string());

        // First, try to match by directory name
        if let Some(name) = &dir_name {
            // Check if we already have a project with this name
            if let Ok(Some(_)) = self.repository.get_project_by_name(name).await {
                return Ok(Some(name.clone()));
            }

            // Check for client project pattern
            let client_name = format!("[CLIENT] {}", name);
            if let Ok(Some(_)) = self.repository.get_project_by_name(&client_name).await {
                return Ok(Some(client_name));
            }
        }

        // Try to get remote URL from git repo
        if let Ok(git_repo) = Repository::open(repo_path) {
            if let Ok(remote) = git_repo.find_remote("origin") {
                if let Some(url) = remote.url() {
                    // Extract repo name from URL
                    if let Some(repo_name) = self.extract_repo_name_from_url(url) {
                        if let Ok(Some(_)) = self.repository.get_project_by_name(&repo_name).await {
                            return Ok(Some(repo_name));
                        }
                    }
                }
            }
        }

        // Return directory name as fallback
        Ok(dir_name)
    }

    /// Extract repository name from git URL
    fn extract_repo_name_from_url(&self, url: &str) -> Option<String> {
        url.split('/')
            .last()
            .map(|name| name.trim_end_matches(".git").to_string())
    }

    /// Create a git time entry from commit analysis
    pub async fn create_git_time_entry(&self, analysis: &CommitAnalysis, project: &Project) -> Result<GitTimeEntry> {
        let confidence_score = self.calculate_confidence_score(analysis);
        
        let git_time_entry = GitTimeEntry::new(
            analysis.commit.hash.clone(),
            project.id,
            project.name.clone(),
            analysis.estimated_duration,
            confidence_score,
        );

        Ok(git_time_entry)
    }

    /// Calculate confidence score for time estimation
    fn calculate_confidence_score(&self, analysis: &CommitAnalysis) -> f32 {
        let mut score: f32 = 0.5; // Base confidence

        // Higher confidence for commits with more context
        if !analysis.commit.message.trim().is_empty() {
            score += 0.2;
        }

        // Higher confidence for commits with reasonable number of changes
        let total_changes = analysis.commit.total_changes() as f32;
        if total_changes > 10.0 && total_changes < 500.0 {
            score += 0.2;
        }

        // Lower confidence for very large commits (might be merges or bulk changes)
        if total_changes > 1000.0 {
            score -= 0.3;
        }

        // Higher confidence for specific commit types
        match analysis.commit_type {
            CommitType::Feature | CommitType::BugFix => score += 0.1,
            _ => {}
        }

        score.max(0.1).min(1.0)
    }

    /// Get recent commits from current directory if it's a git repo
    pub async fn get_recent_commits_from_current_dir(&self, days: u32) -> Result<Vec<GitCommit>> {
        let current_dir = std::env::current_dir()
            .map_err(|e| TimeSpanError::Io(e))?;

        let since = Utc::now() - Duration::days(days as i64);
        self.get_commits(&current_dir, Some(since), Some(50)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::SqliteRepository;
    use std::sync::Arc;
    use std::path::PathBuf;
    

    async fn setup_git_service() -> GitService {
        let repo = Arc::new(SqliteRepository::in_memory().unwrap());
        GitService::new(repo)
    }

    #[tokio::test]
    async fn test_detect_commit_type() {
        let git_commit = GitCommit::new(
            "abc123".to_string(),
            "feat: add new feature".to_string(),
            "Test Author".to_string(),
            "test@example.com".to_string(),
            Utc::now(),
            PathBuf::from("/test"),
        );

        assert_eq!(git_commit.detect_commit_type(), CommitType::Feature);
    }

    #[tokio::test]
    async fn test_calculate_complexity_score() {
        let git_service = setup_git_service().await;
        
        let mut commit = GitCommit::new(
            "abc123".to_string(),
            "test commit".to_string(),
            "Test Author".to_string(),
            "test@example.com".to_string(),
            Utc::now(),
            PathBuf::from("/test"),
        );
        
        commit.insertions = 50;
        commit.deletions = 25;
        commit.files_changed = vec!["file1.rs".to_string(), "file2.rs".to_string()];

        let score = git_service.calculate_complexity_score(&commit);
        assert!(score > 0.0);
        assert!(score < 5.0); // Should be reasonable
    }

    #[tokio::test]
    async fn test_estimate_commit_time() {
        let git_service = setup_git_service().await;
        
        let commit = GitCommit::new(
            "abc123".to_string(),
            "feat: add new feature".to_string(),
            "Test Author".to_string(),
            "test@example.com".to_string(),
            Utc::now(),
            PathBuf::from("/test"),
        );

        let duration = git_service.estimate_commit_time(&commit, &CommitType::Feature, 1.0);
        assert!(duration > Duration::minutes(30));
        assert!(duration < Duration::hours(5));
    }
}
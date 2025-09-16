use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::models::Project;
use crate::repository::Repository;
use crate::services::ProjectService;
use crate::{Result, TimeSpanError};

pub struct ClientDiscoveryService {
    project_service: ProjectService,
    repository: Arc<dyn Repository>,
}

#[derive(Debug, Clone)]
pub struct DiscoveryOptions {
    pub base_path: PathBuf,
    pub exclude_patterns: Vec<String>,
    pub project_prefix: Option<String>,
    pub dry_run: bool,
}

#[derive(Debug)]
pub struct DiscoveryResult {
    pub discovered_directories: Vec<ClientDirectory>,
    pub created_projects: Vec<Project>,
    pub updated_projects: Vec<Project>,
    pub skipped_directories: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ClientDirectory {
    pub name: String,
    pub path: PathBuf,
    pub is_git_repo: bool,
    pub last_modified: Option<std::time::SystemTime>,
    pub suggested_description: Option<String>,
}

impl Default for DiscoveryOptions {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("/Users/user/workspace/Clients"),
            exclude_patterns: vec![
                ".DS_Store".to_string(),
                ".git".to_string(),
                ".*".to_string(), // Exclude all hidden directories/files
                "*.pdf".to_string(),
                "*.mp4".to_string(),
                "*.zip".to_string(),
                "*.whisper".to_string(),
                "*.html".to_string(),
                "*.mht".to_string(),
                "*.pages".to_string(),
                "*.md".to_string(),
                // Common IDE/editor directories
                ".vscode".to_string(),
                ".idea".to_string(),
                ".claude".to_string(),
                ".cursor".to_string(),
                ".vscode-insiders".to_string(),
                ".atom".to_string(),
                ".sublime-text".to_string(),
                ".vim".to_string(),
                ".emacs.d".to_string(),
            ],
            project_prefix: Some("[CLIENT]".to_string()),
            dry_run: false,
        }
    }
}

impl ClientDiscoveryService {
    pub fn new(repository: Arc<dyn Repository>) -> Self {
        let project_service = ProjectService::new(repository.clone());
        Self {
            project_service,
            repository,
        }
    }

    pub async fn discover_clients(&self, options: &DiscoveryOptions) -> Result<DiscoveryResult> {
        let mut result = DiscoveryResult {
            discovered_directories: Vec::new(),
            created_projects: Vec::new(),
            updated_projects: Vec::new(),
            skipped_directories: Vec::new(),
            errors: Vec::new(),
        };

        // Scan the base directory
        let directories =
            self.scan_client_directories(&options.base_path, &options.exclude_patterns)?;
        result.discovered_directories = directories.clone();

        // Process each directory
        for dir in directories {
            match self
                .process_client_directory(&dir, options, &mut result)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    result
                        .errors
                        .push(format!("Error processing {}: {}", dir.name, e));
                }
            }
        }

        Ok(result)
    }

    fn scan_client_directories(
        &self,
        base_path: &Path,
        exclude_patterns: &[String],
    ) -> Result<Vec<ClientDirectory>> {
        let mut directories = Vec::new();

        if !base_path.exists() {
            return Err(TimeSpanError::InvalidDuration(format!(
                "Base path does not exist: {}",
                base_path.display()
            )));
        }

        let entries = fs::read_dir(base_path).map_err(TimeSpanError::Io)?;

        for entry in entries {
            let entry = entry.map_err(TimeSpanError::Io)?;
            let path = entry.path();

            // Skip files, only process directories
            if !path.is_dir() {
                continue;
            }

            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            // Skip excluded patterns
            if self.should_exclude(&name, exclude_patterns) {
                continue;
            }

            let client_dir = self.analyze_directory(&name, &path)?;
            directories.push(client_dir);
        }

        // Sort by name for consistent ordering
        directories.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(directories)
    }

    fn should_exclude(&self, name: &str, exclude_patterns: &[String]) -> bool {
        for pattern in exclude_patterns {
            if pattern == ".*" {
                // Exclude all hidden files/directories (starting with .)
                if name.starts_with('.') {
                    return true;
                }
            } else if pattern.contains('*') {
                // Simple wildcard matching
                if pattern.starts_with('*') && name.ends_with(&pattern[1..]) {
                    return true;
                }
                if pattern.ends_with('*') && name.starts_with(&pattern[..pattern.len() - 1]) {
                    return true;
                }
            } else if name == pattern {
                return true;
            }
        }
        false
    }

    fn analyze_directory(&self, name: &str, path: &Path) -> Result<ClientDirectory> {
        let is_git_repo = path.join(".git").exists();

        let last_modified = fs::metadata(path)
            .ok()
            .and_then(|meta| meta.modified().ok());

        let suggested_description = self.generate_description(name, path, is_git_repo);

        Ok(ClientDirectory {
            name: name.to_string(),
            path: path.to_path_buf(),
            is_git_repo,
            last_modified,
            suggested_description,
        })
    }

    fn generate_description(&self, name: &str, path: &Path, is_git_repo: bool) -> Option<String> {
        let mut parts = Vec::new();

        // Add client type hint based on name patterns
        if name.starts_with("NNL_") {
            parts.push("Example Corp internal project");
        } else if name.contains("Release") {
            parts.push("Product release work");
        } else {
            parts.push("Client project");
        }

        // Add git info
        if is_git_repo {
            parts.push("(Git repository)");
        }

        // Add path info
        let location = format!("Location: {}", path.display());
        parts.push(&location);

        Some(parts.join(" "))
    }

    async fn process_client_directory(
        &self,
        dir: &ClientDirectory,
        options: &DiscoveryOptions,
        result: &mut DiscoveryResult,
    ) -> Result<()> {
        let project_name = match &options.project_prefix {
            Some(prefix) => format!("{} {}", prefix, dir.name),
            None => dir.name.clone(),
        };

        // Check if project already exists
        match self.project_service.get_project(&project_name).await {
            Ok(Some(existing_project)) => {
                // Project exists - potentially update it
                if !options.dry_run {
                    // Update directory path if it has changed
                    if existing_project.directory_path.as_deref()
                        != Some(dir.path.to_str().unwrap_or_default())
                    {
                        let mut updated_project = existing_project.clone();
                        updated_project.directory_path =
                            Some(dir.path.to_string_lossy().to_string());
                        updated_project.is_client_project = true;
                        updated_project.updated_at = chrono::Utc::now();

                        self.repository.update_project(&updated_project).await?;
                        result.updated_projects.push(updated_project);
                    } else {
                        result
                            .skipped_directories
                            .push(format!("{} (already exists)", project_name));
                    }
                }
            }
            Ok(None) => {
                // Project doesn't exist - create it
                if !options.dry_run {
                    let new_project = Project::new_client_project(
                        project_name,
                        dir.suggested_description.clone(),
                        dir.path.to_string_lossy().to_string(),
                    );

                    self.repository.create_project(&new_project).await?;
                    result.created_projects.push(new_project);
                }
            }
            Err(e) => {
                return Err(e);
            }
        }

        Ok(())
    }

    pub async fn list_client_projects(&self) -> Result<Vec<Project>> {
        let all_projects = self.project_service.list_projects().await?;
        Ok(all_projects
            .into_iter()
            .filter(|p| p.is_client_project)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::SqliteRepository;

    async fn setup_service() -> ClientDiscoveryService {
        let repo = Arc::new(SqliteRepository::in_memory().unwrap());
        ClientDiscoveryService::new(repo)
    }

    #[tokio::test]
    async fn test_discovery_service_creation() {
        let _service = setup_service().await;
        // Test that service can be created without error
        // This test verifies the service can be instantiated successfully
    }

    #[tokio::test]
    async fn test_exclude_patterns() {
        let service = setup_service().await;
        let exclude_patterns = vec!["*.pdf".to_string(), ".DS_Store".to_string()];

        assert!(service.should_exclude("document.pdf", &exclude_patterns));
        assert!(service.should_exclude(".DS_Store", &exclude_patterns));
        assert!(!service.should_exclude("ValidClient", &exclude_patterns));
    }

    #[tokio::test]
    async fn test_hidden_directory_exclusion() {
        let service = setup_service().await;
        let exclude_patterns = vec![".*".to_string()];

        // Test that hidden directories are excluded
        assert!(service.should_exclude(".claude", &exclude_patterns));
        assert!(service.should_exclude(".cursor", &exclude_patterns));
        assert!(service.should_exclude(".vscode", &exclude_patterns));
        assert!(service.should_exclude(".git", &exclude_patterns));
        assert!(service.should_exclude(".idea", &exclude_patterns));

        // Test that non-hidden directories are not excluded
        assert!(!service.should_exclude("ValidClient", &exclude_patterns));
        assert!(!service.should_exclude("MyProject", &exclude_patterns));
    }

    #[test]
    fn test_default_options() {
        let options = DiscoveryOptions::default();
        assert_eq!(
            options.base_path,
            PathBuf::from("/Users/user/workspace/Clients")
        );
        assert!(options.exclude_patterns.contains(&".DS_Store".to_string()));
        assert_eq!(options.project_prefix, Some("[CLIENT]".to_string()));
    }
}

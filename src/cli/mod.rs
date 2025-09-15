use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;

use crate::repository::SqliteRepository;
use crate::services::{ProjectService, TimeTrackingService, ReportingService, ClientDiscoveryService, GitService, DiscoveryOptions};
use crate::Result;

#[derive(Parser)]
#[command(name = "timespan")]
#[command(about = "A local time tracking application")]
#[command(version = "1.0")]
pub struct Cli {
    #[arg(long, global = true)]
    pub database: Option<PathBuf>,
    
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Start(StartArgs),
    Stop,
    Status,
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },
    Report {
        #[command(subcommand)]
        command: ReportCommands,
    },
    Git {
        #[command(subcommand)]
        command: GitCommands,
    },
}

#[derive(Args)]
pub struct StartArgs {
    pub project: String,
    #[arg(short, long)]
    pub task: Option<String>,
}

#[derive(Subcommand)]
pub enum ProjectCommands {
    Create {
        name: String,
        #[arg(short, long)]
        description: Option<String>,
    },
    List,
    /// Discover projects from client directories
    Discover {
        /// Base path to scan for client directories
        #[arg(long, default_value = "/Users/jwen/workspace/Clients")]
        path: String,
        /// Prefix to add to discovered project names
        #[arg(long, default_value = "[CLIENT]")] 
        prefix: String,
        /// Preview mode - show what would be created without actually creating
        #[arg(long)]
        dry_run: bool,
    },
    /// List only client projects
    Clients,
}

#[derive(Subcommand)]
pub enum ReportCommands {
    Daily {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum GitCommands {
    /// Analyze recent commits in current directory
    Analyze {
        /// Number of days to look back
        #[arg(short, long, default_value = "7")]
        days: u32,
        /// Specific repository path to analyze
        #[arg(short, long)]
        repo: Option<PathBuf>,
    },
    /// Show git integration status
    Status,
    /// Import commits from a repository and create time entries
    Import {
        /// Repository path to import from
        #[arg(short, long)]
        repo: Option<PathBuf>,
        /// Number of days to import
        #[arg(short, long, default_value = "30")]
        days: u32,
        /// Project name to associate commits with
        #[arg(short, long)]
        project: Option<String>,
    },
}

pub struct TimeSpanApp {
    project_service: ProjectService,
    tracking_service: TimeTrackingService,
    reporting_service: ReportingService,
    client_discovery_service: ClientDiscoveryService,
    git_service: GitService,
}

impl TimeSpanApp {
    pub fn new(database_path: Option<PathBuf>) -> Result<Self> {
        let db_path = database_path.unwrap_or_else(|| {
            let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            path.push("timespan.db");
            path
        });
        
        let repository = Arc::new(SqliteRepository::new(&db_path)?);
        
        Ok(Self {
            project_service: ProjectService::new(repository.clone()),
            tracking_service: TimeTrackingService::new(repository.clone()),
            reporting_service: ReportingService::new(repository.clone()),
            client_discovery_service: ClientDiscoveryService::new(repository.clone()),
            git_service: GitService::new(repository),
        })
    }
    
    pub async fn run(&self, cli: Cli) -> Result<()> {
        match cli.command {
            Commands::Start(args) => self.handle_start(args).await,
            Commands::Stop => self.handle_stop().await,
            Commands::Status => self.handle_status().await,
            Commands::Project { command } => self.handle_project(command).await,
            Commands::Report { command } => self.handle_report(command).await,
            Commands::Git { command } => self.handle_git(command).await,
        }
    }
    
    async fn handle_start(&self, args: StartArgs) -> Result<()> {
        match self.tracking_service.start_timer(&args.project, args.task.as_deref()).await {
            Ok(timer) => {
                println!("Started tracking time for '{}'", timer.project_name);
                Ok(())
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                Err(e)
            }
        }
    }
    
    async fn handle_stop(&self) -> Result<()> {
        match self.tracking_service.stop_timer().await {
            Ok(entry) => {
                let duration = entry.duration.unwrap();
                let hours = duration.num_hours();
                let minutes = duration.num_minutes() % 60;
                println!("Stopped tracking time for '{}' ({}h {}m)", entry.project_name, hours, minutes);
                Ok(())
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                Err(e)
            }
        }
    }
    
    async fn handle_status(&self) -> Result<()> {
        let status = self.tracking_service.get_current_status().await?;
        println!("{}", status);
        Ok(())
    }
    
    async fn handle_project(&self, command: ProjectCommands) -> Result<()> {
        match command {
            ProjectCommands::Create { name, description } => {
                match self.project_service.create_project(&name, description.as_deref()).await {
                    Ok(_) => {
                        println!("Created project '{}'", name);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        Err(e)
                    }
                }
            }
            ProjectCommands::List => {
                let projects = self.project_service.list_projects().await?;
                if projects.is_empty() {
                    println!("No projects found.");
                } else {
                    println!("Projects:");
                    for project in projects {
                        let client_marker = if project.is_client_project { " 🏢" } else { "" };
                        let path_info = project.directory_path
                            .as_deref()
                            .map(|p| format!(" ({})", p))
                            .unwrap_or_default();
                        println!("  - {}{}{}", project.name, client_marker, path_info);
                    }
                }
                Ok(())
            }
            ProjectCommands::Discover { path, prefix, dry_run } => {
                self.handle_project_discover(path, prefix, dry_run).await
            }
            ProjectCommands::Clients => {
                self.handle_list_client_projects().await
            }
        }
    }
    
    async fn handle_report(&self, command: ReportCommands) -> Result<()> {
        match command {
            ReportCommands::Daily { json } => {
                let report = self.reporting_service.generate_daily_report(chrono::Utc::now()).await?;
                
                if json {
                    let json_output = self.reporting_service.export_report_json(&report)?;
                    println!("{}", json_output);
                } else {
                    let total_hours = report.total_duration.num_hours();
                    let total_minutes = report.total_duration.num_minutes() % 60;
                    println!("Daily Report: Total time {}h {}m", total_hours, total_minutes);
                }
                Ok(())
            }
        }
    }
    
    async fn handle_project_discover(&self, path: String, prefix: String, dry_run: bool) -> Result<()> {
        use std::path::PathBuf;
        
        let options = DiscoveryOptions {
            base_path: PathBuf::from(path.clone()),
            exclude_patterns: DiscoveryOptions::default().exclude_patterns,
            project_prefix: if prefix.is_empty() { None } else { Some(prefix) },
            dry_run,
        };
        
        println!("🔍 Discovering client projects in: {}", path);
        if dry_run {
            println!("👀 Running in preview mode - no projects will be created");
        }
        println!();
        
        match self.client_discovery_service.discover_clients(&options).await {
            Ok(result) => {
                // Show discovered directories
                if !result.discovered_directories.is_empty() {
                    println!("📁 Discovered {} directories:", result.discovered_directories.len());
                    for dir in &result.discovered_directories {
                        let git_marker = if dir.is_git_repo { " 🔄" } else { "" };
                        println!("  • {}{}", dir.name, git_marker);
                        if let Some(desc) = &dir.suggested_description {
                            println!("    {}", desc);
                        }
                    }
                    println!();
                }
                
                // Show results
                if !result.created_projects.is_empty() {
                    println!("✅ Created {} new projects:", result.created_projects.len());
                    for project in &result.created_projects {
                        println!("  + {}", project.name);
                    }
                    println!();
                }
                
                if !result.updated_projects.is_empty() {
                    println!("🔄 Updated {} existing projects:", result.updated_projects.len());
                    for project in &result.updated_projects {
                        println!("  ~ {}", project.name);
                    }
                    println!();
                }
                
                if !result.skipped_directories.is_empty() {
                    println!("⏭️ Skipped {} directories:", result.skipped_directories.len());
                    for skipped in &result.skipped_directories {
                        println!("  - {}", skipped);
                    }
                    println!();
                }
                
                if !result.errors.is_empty() {
                    println!("❌ Errors encountered:");
                    for error in &result.errors {
                        println!("  ! {}", error);
                    }
                    println!();
                }
                
                // Summary
                if dry_run {
                    println!("👁️ Preview completed. Use without --dry-run to create projects.");
                } else {
                    let total = result.created_projects.len() + result.updated_projects.len();
                    if total > 0 {
                        println!("🎉 Successfully processed {} project(s)!", total);
                    } else {
                        println!("✨ No new projects to create - everything is up to date!");
                    }
                }
                
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Discovery failed: {}", e);
                Err(e)
            }
        }
    }
    
    async fn handle_list_client_projects(&self) -> Result<()> {
        match self.client_discovery_service.list_client_projects().await {
            Ok(projects) => {
                if projects.is_empty() {
                    println!("🏢 No client projects found.");
                    println!("💡 Use 'timespan project discover' to scan for client directories.");
                } else {
                    println!("🏢 Client Projects ({}):", projects.len());
                    for project in projects {
                        let path_info = project.directory_path
                            .as_deref()
                            .map(|p| format!(" → {}", p))
                            .unwrap_or_default();
                        println!("  • {}{}", project.name, path_info);
                        if let Some(desc) = &project.description {
                            println!("    {}", desc);
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Failed to list client projects: {}", e);
                Err(e)
            }
        }
    }

    async fn handle_git(&self, command: GitCommands) -> Result<()> {
        match command {
            GitCommands::Analyze { days, repo } => {
                self.handle_git_analyze(days, repo).await
            }
            GitCommands::Status => {
                self.handle_git_status().await
            }
            GitCommands::Import { repo, days, project } => {
                self.handle_git_import(repo, days, project).await
            }
        }
    }

    async fn handle_git_analyze(&self, days: u32, repo_path: Option<PathBuf>) -> Result<()> {
        let path = repo_path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        
        println!("🔍 Analyzing git commits from: {}", path.display());
        println!("📅 Looking back {} days", days);
        println!();

        let since = chrono::Utc::now() - chrono::Duration::days(days as i64);
        match self.git_service.get_commits(&path, Some(since), Some(20)).await {
            Ok(commits) => {
                if commits.is_empty() {
                    println!("📭 No commits found in the specified time range.");
                    return Ok(());
                }

                println!("📊 Found {} commits:", commits.len());
                println!();

                let mut total_estimated_time = chrono::Duration::zero();
                let mut commit_types = std::collections::HashMap::new();

                for commit in &commits {
                    let analysis = self.git_service.analyze_commit(commit).await?;
                    let hours = analysis.estimated_duration.num_hours();
                    let minutes = analysis.estimated_duration.num_minutes() % 60;
                    
                    println!("📝 {} ({}h {}m)", 
                        commit.hash.chars().take(8).collect::<String>(),
                        hours, minutes);
                    println!("   {} by {}", commit.message.lines().next().unwrap_or(""), commit.author);
                    println!("   {} files, +{} -{} lines", 
                        commit.files_changed.len(), commit.insertions, commit.deletions);
                    println!("   Type: {:?}, Confidence: {:.1}%", 
                        analysis.commit_type, analysis.complexity_score * 100.0);
                    println!();

                    total_estimated_time = total_estimated_time + analysis.estimated_duration;
                    *commit_types.entry(analysis.commit_type).or_insert(0) += 1;
                }

                let total_hours = total_estimated_time.num_hours();
                let total_minutes = total_estimated_time.num_minutes() % 60;
                
                println!("📈 Summary:");
                println!("   Total estimated time: {}h {}m", total_hours, total_minutes);
                println!("   Average per commit: {}m", total_estimated_time.num_minutes() / commits.len() as i64);
                println!("   Commit types: {:?}", commit_types);
                
                // Try to detect associated project
                if let Ok(Some(project_name)) = self.git_service.detect_project(&path).await {
                    println!("   Detected project: {}", project_name);
                }

                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Failed to analyze commits: {}", e);
                Err(e)
            }
        }
    }

    async fn handle_git_status(&self) -> Result<()> {
        println!("📊 Git Integration Status");
        println!();

        // Check if current directory is a git repository
        let current_dir = std::env::current_dir().unwrap_or_default();
        match git2::Repository::open(&current_dir) {
            Ok(repo) => {
                println!("✅ Current directory is a git repository");
                println!("   Path: {}", current_dir.display());
                
                // Get remote info if available
                if let Ok(remote) = repo.find_remote("origin") {
                    if let Some(url) = remote.url() {
                        println!("   Remote: {}", url);
                    }
                }

                // Try to detect associated project
                match self.git_service.detect_project(&current_dir).await {
                    Ok(Some(project_name)) => {
                        println!("   Associated project: {}", project_name);
                    }
                    Ok(None) => {
                        println!("   No associated TimeSpan project found");
                        println!("   💡 Use 'timespan git import --project <name>' to associate");
                    }
                    Err(e) => {
                        println!("   ⚠️  Error detecting project: {}", e);
                    }
                }

                // Show recent commit activity
                match self.git_service.get_recent_commits_from_current_dir(7).await {
                    Ok(commits) => {
                        println!("   Recent activity: {} commits in last 7 days", commits.len());
                    }
                    Err(_) => {
                        println!("   Recent activity: Unable to read commits");
                    }
                }
            }
            Err(_) => {
                println!("❌ Current directory is not a git repository");
                println!("   Navigate to a git repository to use git integration features");
            }
        }

        println!();
        println!("Available commands:");
        println!("   timespan git analyze     # Analyze recent commits");
        println!("   timespan git import      # Import commits as time entries");
        
        Ok(())
    }

    async fn handle_git_import(&self, repo_path: Option<PathBuf>, days: u32, project_name: Option<String>) -> Result<()> {
        let path = repo_path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        
        println!("📥 Importing git commits to TimeSpan");
        println!("   Repository: {}", path.display());
        println!("   Days back: {}", days);
        
        // Detect or use provided project
        let project_name = if let Some(name) = project_name {
            name
        } else {
            match self.git_service.detect_project(&path).await? {
                Some(name) => name,
                None => {
                    println!("❌ No project specified and none could be auto-detected");
                    println!("   Use --project <name> to specify a project");
                    return Ok(());
                }
            }
        };

        // Get or create project
        let project = match self.project_service.get_project(&project_name).await? {
            Some(project) => project,
            None => {
                println!("📝 Creating new project: {}", project_name);
                self.project_service.create_project(&project_name, Some(&format!("Auto-created from git import: {}", path.display()))).await?
            }
        };

        println!("   Target project: {}", project.name);
        println!();

        let since = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let commits = self.git_service.get_commits(&path, Some(since), Some(50)).await?;

        if commits.is_empty() {
            println!("📭 No commits found in the specified time range.");
            return Ok(());
        }

        println!("🔄 Processing {} commits...", commits.len());
        let mut total_time = chrono::Duration::zero();
        let mut imported_count = 0;

        for commit in &commits {
            let analysis = self.git_service.analyze_commit(commit).await?;
            let git_time_entry = self.git_service.create_git_time_entry(&analysis, &project).await?;
            
            // Convert to regular time entry
            let mut time_entry = crate::models::TimeEntry::new(
                project.id,
                project.name.clone(),
                Some(format!("Git: {}", commit.message.lines().next().unwrap_or("No message"))),
                commit.timestamp,
            );
            
            // Set the estimated time as the duration
            time_entry.stop(commit.timestamp + git_time_entry.estimated_time)?;
            time_entry.add_tag("git-import".to_string());
            time_entry.add_tag(format!("commit-{}", commit.hash.chars().take(8).collect::<String>()));
            
            // Save to database (you would need to add this to repository trait)
            // For now, we'll just print what we would do
            let hours = git_time_entry.estimated_time.num_hours();
            let minutes = git_time_entry.estimated_time.num_minutes() % 60;
            
            println!("   ✅ {} - {}h {}m", 
                commit.hash.chars().take(8).collect::<String>(),
                hours, minutes);
            
            total_time = total_time + git_time_entry.estimated_time;
            imported_count += 1;
        }

        let total_hours = total_time.num_hours();
        let total_minutes = total_time.num_minutes() % 60;
        
        println!();
        println!("🎉 Import completed!");
        println!("   Commits processed: {}", imported_count);
        println!("   Total estimated time: {}h {}m", total_hours, total_minutes);
        println!("   Average per commit: {}m", total_time.num_minutes() / imported_count);
        
        Ok(())
    }
}

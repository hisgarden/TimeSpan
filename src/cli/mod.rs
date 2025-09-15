use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;

use crate::repository::SqliteRepository;
use crate::services::{ProjectService, TimeTrackingService, ReportingService, ClientDiscoveryService, DiscoveryOptions};
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

pub struct TimeSpanApp {
    project_service: ProjectService,
    tracking_service: TimeTrackingService,
    reporting_service: ReportingService,
    client_discovery_service: ClientDiscoveryService,
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
            client_discovery_service: ClientDiscoveryService::new(repository),
        })
    }
    
    pub async fn run(&self, cli: Cli) -> Result<()> {
        match cli.command {
            Commands::Start(args) => self.handle_start(args).await,
            Commands::Stop => self.handle_stop().await,
            Commands::Status => self.handle_status().await,
            Commands::Project { command } => self.handle_project(command).await,
            Commands::Report { command } => self.handle_report(command).await,
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
}

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;

use crate::repository::SqliteRepository;
use crate::services::{ProjectService, TimeTrackingService, ReportingService};
use crate::{Result, TimeSpanError};

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
            reporting_service: ReportingService::new(repository),
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
                        println!("  - {}", project.name);
                    }
                }
                Ok(())
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
}
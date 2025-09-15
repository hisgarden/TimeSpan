pub mod models;
pub mod repository;
pub mod services;
pub mod cli;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TogglError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Timer is already running for project: {0}")]
    TimerAlreadyRunning(String),
    #[error("No active timer found")]
    NoActiveTimer,
    #[error("Project not found: {0}")]
    ProjectNotFound(String),
    #[error("Project already exists: {0}")]
    ProjectAlreadyExists(String),
    #[error("Cannot delete project with time entries: {0}")]
    ProjectHasTimeEntries(String),
    #[error("Invalid duration format: {0}")]
    InvalidDuration(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, TogglError>;
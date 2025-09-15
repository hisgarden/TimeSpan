# Toggl Local - Rust Time Tracker

A local time tracking application built with Rust using Test-Driven Development (TDD) and Behavior-Driven Development (BDD) methodologies.

## Features

- ✅ **Project Management**: Create, list, update, and delete projects
- ✅ **Time Tracking**: Start/stop timers with task descriptions  
- ✅ **Status Monitoring**: View current timer status and elapsed time
- ✅ **Reporting**: Generate daily, weekly, and project-specific reports
- ✅ **Data Export**: Export reports as JSON
- ✅ **Tag Support**: Add tags to time entries for better categorization
- ✅ **Local Storage**: All data stored locally in SQLite database
- ✅ **CLI Interface**: Full-featured command-line interface

## Quick Start

```bash
# Build the project
cargo build --release

# Create a project
./target/release/toggl project create "My Project"

# Start tracking time
./target/release/toggl start "My Project" --task "Working on something"

# Check status
./target/release/toggl status

# Stop tracking
./target/release/toggl stop

# View daily report
./target/release/toggl report daily
```

## Architecture

The project follows clean architecture principles:

- **Models**: Domain entities (Project, TimeEntry, Timer)
- **Repository**: Data access layer with SQLite
- **Services**: Business logic layer
- **CLI**: Command-line interface

## Testing

Built with TDD/BDD - 32 comprehensive tests covering all functionality:

```bash
cargo test  # Run all tests
```

## Development

This project demonstrates:
- Test-Driven Development practices
- Clean architecture in Rust
- Async programming with tokio
- CLI development with clap
- SQLite database operations
- Error handling with custom types

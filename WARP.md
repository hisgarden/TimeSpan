# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

TimeSpan is a local time tracking application built with Rust using Test-Driven Development (TDD) and Behavior-Driven Development (BDD) methodologies. It's designed to help developers track time spent on projects with comprehensive CLI interface, local SQLite storage, and project management features including client discovery.

## Architecture

The project follows clean architecture principles with clear separation of concerns:

### Core Architecture Layers
- **Models** (`src/models/mod.rs`): Domain entities (Project, TimeEntry, Timer, TimeReport) with business logic
- **Repository** (`src/repository/mod.rs`): Data access layer with async trait and SQLite implementation
- **Services** (`src/services/mod.rs`): Business logic services (ProjectService, TimeTrackingService, ReportingService, ClientDiscoveryService)  
- **CLI** (`src/cli/mod.rs`): Command-line interface using clap with structured commands

### Key Domain Concepts
- **Projects**: Can be regular projects or client projects (with directory paths)
- **Timers**: Active time tracking state stored separately from completed entries
- **TimeEntry**: Completed time tracking records with duration and tags
- **Client Discovery**: Automated scanning of client directories to create projects

### Data Flow
1. CLI parses commands and delegates to TimeSpanApp
2. TimeSpanApp routes to appropriate service (Project/Tracking/Reporting/Discovery)
3. Services contain business logic and coordinate with repository
4. Repository handles all SQLite database operations with async trait abstraction

## Common Development Commands

### Build and Run
```bash
cargo build --release              # Build optimized binary
cargo run -- --help               # Show CLI help
```

### Testing
```bash
cargo test                         # Run all unit and integration tests (32 tests total)
cargo test --test cucumber         # Run BDD/Cucumber integration tests specifically
cargo test models::tests          # Run model unit tests
cargo test repository::tests      # Run repository tests
cargo test services::tests        # Run service tests
```

### Running the Application
```bash
# Basic project and time tracking workflow
./target/release/timespan project create "My Project"
./target/release/timespan start "My Project" --task "Working on feature"
./target/release/timespan status
./target/release/timespan stop
./target/release/timespan report daily --json

# Client project discovery
./target/release/timespan project discover --path /Users/user/workspace/Clients --dry-run
./target/release/timespan project clients
```

### Database Management
The SQLite database (`timespan.db`) is created automatically in the current directory. The repository handles schema migrations automatically.

## Testing Architecture

### Test Structure
- **Unit Tests**: Embedded in each module (`#[cfg(test)]` blocks)
- **Integration Tests**: BDD scenarios using Cucumber framework (`features/` directory)
- **Test Utilities**: In-memory SQLite database for isolated testing

### BDD Features
- `features/project_management.feature`: Project CRUD operations
- `features/time_tracking.feature`: Timer start/stop, status checking
- `features/reporting.feature`: Report generation and export

### Running Specific Test Scenarios
```bash
# Run specific test functions
cargo test test_create_project
cargo test test_start_and_stop_timer
cargo test test_client_discovery

# Test with verbose output
cargo test -- --nocapture
```

## Code Patterns and Conventions

### Error Handling
- Custom error types defined in `TimeSpanError` enum with thiserror
- Repository operations return `Result<T>` with proper error propagation
- Database constraint violations mapped to domain-specific errors

### Async Patterns
- Repository trait is fully async with `#[async_trait]`
- Services coordinate async repository calls
- No blocking operations in business logic layer

### Database Schema Migrations
The repository automatically handles schema migrations in `migrate_database_schema()`. New columns are added safely with ALTER TABLE statements.

### Client Project Discovery
Special feature for automatically discovering client projects from directory structure:
- Scans for git repositories and subdirectories
- Creates projects with `[CLIENT]` prefix and directory path metadata
- Supports dry-run mode for preview before creation

## Development Workflow

When adding new features:

1. **Write failing tests first** (TDD approach)
2. **Add BDD scenarios** to `features/` directory if user-facing
3. **Update models** if new domain concepts are needed
4. **Extend repository trait** for new data operations
5. **Implement in repository** with proper error handling
6. **Add service methods** for business logic
7. **Wire up CLI commands** in `src/cli/mod.rs`

## Important Implementation Details

- **Timer vs TimeEntry**: Active timers are stored separately from completed time entries
- **Project uniqueness**: Enforced at database level with unique constraint on name
- **Tag system**: JSON serialization for flexible tag storage
- **Time calculations**: Uses chrono::Duration for all time arithmetic
- **Database transactions**: SQLite operations are atomic but not explicitly transactional
- **Client project detection**: Based on directory structure and git repository presence

## User Preferences

The user prefers to alias 'docker' to 'podman' and 'docker-compose' to 'podman-compose' in their shell configuration.
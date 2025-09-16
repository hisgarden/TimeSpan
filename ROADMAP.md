# TimeSpan Roadmap

## 🎯 Vision
Evolve TimeSpan from a manual time tracker into a Git-integrated productivity analytics platform.

## 🗺️ Roadmap Overview

### Phase 1: Git Integration & Automation (v1.1.0)
**Duration**: 2-3 weeks  
**Priority**: Highest  

### Phase 2: Enhanced Time Tracking (v1.2.0) 
**Duration**: 3-4 weeks  
**Priority**: High  

### Phase 3: Productivity & Analytics (v1.3.0)
**Duration**: 4-5 weeks  
**Priority**: Medium  

### Phase 4: User Experience (v1.4.0)
**Duration**: 3-4 weeks  
**Priority**: Medium  

### Phase 5: Customization (v1.5.0)
**Duration**: 2-3 weeks  
**Priority**: Low  

---

## 🔄 Phase 1: Git Integration & Automation (v1.1.0)

### Feature 1.1: Git Commit Auto-Tracking

#### 🎯 **Goals**
- Automatically record time entries based on git commits
- Eliminate manual timer management
- Provide realistic work progress tracking
- Support multiple repositories and projects

#### 🏗️ **Architecture Changes**

**New Services**:
- `GitService` - Git repository operations and analysis
- `GitHookService` - Git hook management and installation
- `CommitAnalysisService` - Parse and analyze commit data

**New Models**:
```rust
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub files_changed: Vec<String>,
    pub insertions: u32,
    pub deletions: u32,
    pub repository_path: PathBuf,
}

pub struct GitTimeEntry {
    pub commit_hash: String,
    pub project_id: Uuid,
    pub estimated_time: Duration,
    pub actual_time: Option<Duration>,
    pub confidence_score: f32,
}
```

**New Commands**:
```bash
timespan git setup                           # Install git hooks globally
timespan git setup --repo /path/to/repo     # Install for specific repo
timespan git configure                       # Configure git integration settings
timespan git analyze                         # Analyze existing commit history
timespan git sync                           # Sync commits to time entries
timespan git status                         # Show git integration status
```

#### 🔧 **Implementation Steps**

**Step 1: Git Operations**
```rust
// src/services/git_service.rs
impl GitService {
    pub async fn get_commits(&self, repo_path: &Path, since: Option<DateTime<Utc>>) -> Result<Vec<GitCommit>>
    pub async fn analyze_commit(&self, commit: &GitCommit) -> Result<CommitAnalysis>
    pub async fn estimate_time(&self, analysis: &CommitAnalysis) -> Result<Duration>
    pub async fn detect_project(&self, repo_path: &Path) -> Result<Option<String>>
}
```

**Step 2: Git Hook System**
```bash
#!/bin/sh
# .git/hooks/post-commit
timespan git record-commit --repo $(pwd) --commit $(git rev-parse HEAD)
```

**Step 3: Time Estimation Algorithm**
```rust
pub fn estimate_commit_time(analysis: &CommitAnalysis) -> Duration {
    let mut base_time = Duration::minutes(15); // Base time per commit
    
    // Adjust based on lines changed
    let lines_factor = (analysis.insertions + analysis.deletions) as f32 / 50.0;
    base_time = base_time + Duration::minutes((lines_factor * 10.0) as i64);
    
    // Adjust based on file types
    for file in &analysis.files_changed {
        match file.extension() {
            Some("rs") => base_time += Duration::minutes(5),  // Rust files
            Some("js") | Some("ts") => base_time += Duration::minutes(3),
            Some("md") => base_time += Duration::minutes(1),
            _ => {}
        }
    }
    
    // Adjust based on commit message patterns
    if analysis.message.contains("fix") || analysis.message.contains("bug") {
        base_time += Duration::minutes(10); // Bug fixes take longer
    }
    
    base_time.min(Duration::hours(4)) // Cap at 4 hours per commit
}
```

**Step 4: Configuration System**
```toml
# ~/.timespan/git-config.toml
[git]
auto_track = true
default_minutes_per_commit = 30
max_hours_per_commit = 4

[estimation]
base_time_minutes = 15
lines_per_minute = 2
bug_fix_bonus_minutes = 10

[file_weights]
"*.rs" = 1.5
"*.js" = 1.2
"*.ts" = 1.2
"*.md" = 0.5
"*.json" = 0.3

[project_detection]
use_directory_name = true
use_git_remote = true
fallback_project = "General Development"
```

### Feature 1.2: Git-Based Reports

#### 📊 **New Report Types**

**Commit Velocity Report**:
```bash
timespan report commits --period week
# Output:
# Git Commit Analysis (Last 7 days)
# Total Commits: 23
# Estimated Time: 12.5 hours
# Average per Commit: 32 minutes
# Most Active Day: Tuesday (8 commits)
# Repository Breakdown:
#   - TimeSpan: 15 commits (8.2h)
#   - Client Work: 8 commits (4.3h)
```

**Repository Activity**:
```bash
timespan report repos --top 10
# Shows top 10 repositories by time spent
```

### Feature 1.3: Retrospective Analysis

**Analyze Historical Commits**:
```bash
timespan git import --since "2024-01-01"  # Import all commits since date
timespan git import --repo /path/to/repo   # Import specific repository
```

#### 🧪 **Testing Strategy**

**Unit Tests**:
- Git commit parsing and analysis
- Time estimation algorithms
- Project detection logic

**Integration Tests**:
- Git hook installation and execution
- End-to-end commit tracking flow
- Multiple repository scenarios

**Test Repositories**:
- Create test git repositories with known commit patterns
- Validate time estimation accuracy
- Test edge cases (large commits, merge commits, etc.)

---

## 🚀 Quick Start Implementation

### Immediate Next Steps (Week 1):

1. **Create Git Service Foundation**
   - Add `git2` dependency to Cargo.toml
   - Create `src/services/git_service.rs`
   - Implement basic commit reading functionality

2. **Design Data Models** 
   - Add GitCommit struct to models
   - Extend database schema for git integration
   - Create migration scripts

3. **Basic CLI Commands**
   - Add `git` subcommand to CLI
   - Implement `timespan git status` and `timespan git analyze`

4. **Proof of Concept**
   - Create simple post-commit hook
   - Test basic time estimation
   - Validate project detection

Would you like me to start implementing the Git integration feature? This would be a useful addition that makes TimeSpan more practical for real-world development work.
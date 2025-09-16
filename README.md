# TimeSpan ⏱️

**Local Time Tracking for Developers**

TimeSpan is a privacy-first time tracking application designed for developers who value accuracy, automation, and local data control. Track your work with Git integration, project detection, and comprehensive reporting.

## ✨ Key Features

### 🚀 **Automatic Git Integration**
- **Commit Analysis**: Automatically estimates time based on commit complexity
- **Retroactive Tracking**: Import historical work from Git commits
- **One-Command Workflow**: `ts` command combines commit + time tracking
- **Automatic Classification**: Automatically categorizes commits (Feature, BugFix, Test, etc.)

### 📁 **Project Management** 
- **Client Project Discovery**: Auto-detect projects from directory structures
- **Project Organization**: Separate personal and client work automatically
- **Flexible Structure**: Support for complex project hierarchies

### ⏰ **Comprehensive Time Tracking**
- **Manual Timers**: Traditional start/stop time tracking
- **Task Descriptions**: Add context to your work sessions
- **Status Monitoring**: Always know what you're working on
- **Local Data**: Everything stored securely on your machine
- **Centralized Database**: All data stored in XDG-compliant location (`~/.local/share/timespan/` on Linux, `~/Library/Application Support/timespan/` on macOS)

### 📊 **Detailed Reporting**
- **Daily Reports**: See your daily work breakdown
- **JSON Export**: Integrate with other tools and systems
- **Project Summaries**: Track time across different projects
- **Historical Analysis**: Understand your work patterns

## 📥 Installation

### ⚡ One-Line Install (Recommended)

**Install TimeSpan + optional Git integration setup:**

```bash
curl -fsSL https://raw.githubusercontent.com/hisgarden/TimeSpan/main/install.sh | bash
```

### 🍺 Just Homebrew

**If you prefer just the binary:**

```bash
brew tap hisgarden/timespan && brew install timespan
```

That's it! TimeSpan is ready to use with full Git integration.

## 🗄️ Database Location

TimeSpan stores all your data in a centralized location following the XDG Base Directory Specification:

- **macOS**: `~/Library/Application Support/timespan/timespan.db`
- **Linux**: `~/.local/share/timespan/timespan.db`
- **Windows**: `%APPDATA%/timespan/timespan.db`

### Custom Database Location

You can specify a custom database location using the `--database` flag:

```bash
# Use a custom database file
timespan --database /path/to/custom.db project list

# Use a project-specific database
timespan --database ./project.db start "My Project"
```

<details>
<summary>🔧 Alternative Installation Methods</summary>

### From Source (for developers)

```bash
# Clone and install
git clone https://github.com/hisgarden/TimeSpan.git
cd TimeSpan
cargo install --path .
```

### Manual Download

1. Download the latest release from [GitHub Releases](https://github.com/hisgarden/TimeSpan/releases)
2. Extract and add to your PATH

</details>

## 🚀 Quick Start

### Traditional Time Tracking

```bash
# Create a project
timespan project create "My Project"

# Start tracking time
timespan start "My Project" --task "Working on authentication"

# Check current status
timespan status
# Output: ⏱️  My Project (0h 15m) - Working on authentication

# Stop tracking
timespan stop
# Output: Stopped tracking time for 'My Project' (0h 15m)

# View daily report
timespan report daily
```

### 🎆 **Git Integration**

The `ts` command combines Git commits with automatic time tracking:

```bash
# Navigate to any Git repository
cd /path/to/your/project

# Make your changes, then commit + track time in one command
ts implemented user authentication with JWT tokens

# TimeSpan automatically:
# ✅ Commits your changes to Git
# ✅ Analyzes the commit complexity
# ✅ Estimates time based on files/lines changed
# ✅ Imports the time entry to your TimeSpan database
# ✅ Detects if it's a client project automatically
```

**Example Output:**
```
🏢 Detected client project: [CLIENT] Acme Corp
📝 Adding changes to git...
💾 Committing: implemented user authentication with JWT tokens
[main a1b2c3d] implemented user authentication with JWT tokens
 8 files changed, 342 insertions(+), 15 deletions(-)
✅ Git commit successful!
🔍 Running TimeSpan git analysis...
📊 Found 1 commit:
📝 a1b2c3d (2h 15m) - Type: Feature, Confidence: 87.5%
✅ Time automatically tracked based on commit!
```

## 📚 Complete Command Reference

### Project Management

```bash
# Create a new project
timespan project create "Website Redesign" --description "Complete site overhaul"

# List all projects
timespan project list

# Discover client projects from directories
timespan project discover --path /Users/me/workspace/Clients

# Preview discovery without creating (dry run)
timespan project discover --path /Users/me/workspace/Clients --dry-run

# List only client projects
timespan project clients
```

### Time Tracking

```bash
# Start a timer
timespan start "Website Redesign" --task "Homepage mockups"

# Check what you're currently working on
timespan status

# Stop the current timer
timespan stop
```

### Git Integration

```bash
# Analyze recent commits in current directory
timespan git analyze

# Analyze last 14 days
timespan git analyze --days 14

# Check Git integration status
timespan git status

# Import commits as time entries
timespan git import --project "My Project" --days 7

# The magic 'ts' command (requires setup)
ts fixed critical bug in payment processing
```

### Reporting

```bash
# View today's work summary
timespan report daily

# Export daily report as JSON for integration
timespan report daily --json

# Save report to file
timespan report daily --json > today_report.json
```

## 🔧 Setup the `ts` Magic Command (Optional)

Unlock the ultimate developer experience with the `ts` command that combines Git commits + automatic time tracking:

> 💫 **Pro tip:** If you used the one-line installer, this might already be set up for you!

### Manual Setup (30 seconds)

1. **Download the integration script:**
   ```bash
   curl -fsSL https://raw.githubusercontent.com/hisgarden/TimeSpan/main/.timespan_warp_integration.zsh -o ~/.timespan_warp_integration.zsh
   ```

2. **Add one line to your shell config** (`.zshrc` or `.bashrc`):
   ```bash
   echo 'source ~/.timespan_warp_integration.zsh' >> ~/.zshrc
   ```

3. **Reload your shell:**
   ```bash
   source ~/.zshrc
   ```

### Now Use Anywhere! ✨

```bash
cd /any/git/repository
ts completed user dashboard with real-time updates
# ✅ Commits to Git + tracks time automatically!
```

**What the `ts` command does:**
- 📝 Commits your changes to Git with your message
- 🤖 Analyzes commit complexity automatically  
- ⏱️ Estimates realistic time based on changes
- 🏢 Detects client projects automatically
- 📊 Logs time entry to TimeSpan database

## 📊 Understanding Reports

### Daily Report Example

```
Daily Report: Total time 6h 23m

Project Summaries:
  Website Redesign          4h 15m  (3 sessions)
  [CLIENT] Acme Corp       2h 8m   (2 sessions)

Detailed Entries:
  09:00-11:15  Website Redesign     "Homepage mockups"
  13:30-15:45  [CLIENT] Acme Corp   "API integration"
  16:00-18:00  Website Redesign     "Responsive design"
```

### JSON Export Structure

```json
{
  "total_duration": [22980, 0],
  "entries": [
    {
      "project_name": "Website Redesign",
      "task_description": "Homepage mockups",
      "start_time": "2023-10-15T09:00:00Z",
      "end_time": "2023-10-15T11:15:00Z",
      "duration": [8100, 0]
    }
  ],
  "project_summaries": [
    {
      "project_name": "Website Redesign",
      "total_duration": [15300, 0],
      "entry_count": 2
    }
  ]
}
```

## 🔒 Privacy & Data

- **Local First**: All data stored locally in SQLite
- **No Cloud**: Your time data never leaves your machine
- **Portable**: Database file can be backed up/synced as needed
- **Secure**: No accounts, no tracking, no external dependencies

## ❓ FAQ

**Q: Where is my data stored?**  
A: In a local SQLite database (`timespan.db`) in your current directory or specified location.

**Q: Can I use this without Git?**  
A: Absolutely! The traditional start/stop timer functionality works independently.

**Q: Does the Git integration work with any repository?**  
A: Yes, it works with any Git repository on your local machine.

**Q: Can I export my data?**  
A: Yes, use `timespan report daily --json` to export in JSON format.

**Q: How accurate is the Git-based time estimation?**  
A: The algorithm considers file complexity, lines changed, and commit patterns. It's designed to be realistic rather than perfect.

---

**TimeSpan v1.1.0** - Time tracking for developers ✨

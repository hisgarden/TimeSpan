# TimeSpan Warp Integration
# Usage: ts work done on client XXX
# This will commit changes to git and automatically track time in TimeSpan

ts() {
    # Check if we're in a git repository
    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "❌ Error: Not in a git repository"
        return 1
    fi
    
    # Check if there are changes to commit (including untracked files)
    if git diff --quiet && git diff --cached --quiet && [ -z "$(git ls-files --others --exclude-standard)" ]; then
        echo "❌ No changes to commit"
        return 1
    fi
    
    # Get the commit message from arguments
    if [ $# -eq 0 ]; then
        echo "❌ Usage: ts <commit message>"
        echo "Example: ts work done on client CSAR authentication"
        return 1
    fi
    
    local commit_msg="$*"
    local current_dir=$(pwd)
    local project_name=""
    
    # Try to detect client project from path
    if [[ "$current_dir" == *"/Clients/"* ]]; then
        # Extract client name from path
        local client_path=$(echo "$current_dir" | sed 's/.*\/Clients\///' | cut -d'/' -f1)
        project_name="[CLIENT] $client_path"
        echo "🏢 Detected client project: $project_name"
    else
        # For non-client projects, use directory name
        project_name=$(basename "$current_dir")
        echo "📁 Detected project: $project_name"
    fi
    
    # Add all changes
    echo "📝 Adding changes to git..."
    git add .
    
    # Commit with the message
    echo "💾 Committing: $commit_msg"
    if git commit -m "$commit_msg"; then
        echo "✅ Git commit successful!"
        
        # Check if timespan is available
        if command -v timespan >/dev/null 2>&1; then
            local timespan_cmd="timespan"
            echo "⏱️  Analyzing commit for time tracking..."
            
            # Try to import the latest commit into TimeSpan
            # First check if git commands are available in timespan
            if $timespan_cmd git analyze >/dev/null 2>&1; then
                # Git integration is available
                echo "🔍 Running TimeSpan git analysis..."
                $timespan_cmd git analyze --days 1
                
                echo "📊 Importing time entry..."
                $timespan_cmd git import --project "$project_name" --days 1
                
                echo "✅ Time automatically tracked based on commit!"
            else
                # Fallback to manual time estimation
                echo "⚠️  Git integration not available, using manual estimation..."
                
                # Simple estimation based on commit size
                local files_changed=$(git diff --name-only HEAD~1 HEAD | wc -l | tr -d ' ')
                local lines_changed=$(git diff --shortstat HEAD~1 HEAD | grep -o '[0-9]* insertion' | head -1 | grep -o '[0-9]*')
                
                if [ -z "$lines_changed" ]; then
                    lines_changed=0
                fi
                
                # Simple estimation: 1 minute per file + 1 minute per 10 lines
                local estimated_minutes=$((files_changed + lines_changed / 10))
                if [ $estimated_minutes -lt 5 ]; then
                    estimated_minutes=5  # Minimum 5 minutes
                fi
                
                echo "📈 Estimated work time: ${estimated_minutes} minutes"
                echo "   (${files_changed} files, ${lines_changed} lines changed)"
                
                # Create a time entry (this would need TimeSpan to support direct time entry creation)
                echo "💡 Tip: Run 'timespan start \"$project_name\" --task \"$commit_msg\"' before work"
                echo "       Then 'timespan stop' when done for accurate tracking"
            fi
            
            # Show current status
            echo "📊 Current TimeSpan status:"
            $timespan_cmd status
        else
            echo "⚠️  TimeSpan not found in PATH"
            echo "💡 Install with: brew tap hisgarden/timespan && brew install timespan"
        fi
        
        # Show git log for confirmation
        echo ""
        echo "📋 Latest commits:"
        git log --oneline -3
        
    else
        echo "❌ Git commit failed!"
        return 1
    fi
}

# Auto-completion for ts command (basic)
# Only set up completion if compdef is available
if command -v compdef >/dev/null 2>&1; then
    _ts_completion() {
        _message "commit message (e.g., work done on client authentication)"
    }
    
    # Register completion
    compdef _ts_completion ts
fi

echo "✅ TimeSpan Warp integration loaded!"
echo "💡 Usage: ts work done on client authentication"
echo "🔧 Type 'ts' without arguments for help"
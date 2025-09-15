#!/bin/bash

# TimeSpan One-Line Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/hisgarden/TimeSpan/main/install.sh | bash

set -e

echo "🚀 Installing TimeSpan - Intelligent Time Tracking for Developers"
echo

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    echo "❌ Homebrew is required but not installed."
    echo "📥 Install Homebrew first: https://brew.sh"
    echo "   /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
    exit 1
fi

echo "📦 Installing TimeSpan via Homebrew..."

# Add tap and install
brew tap hisgarden/timespan
brew install timespan

echo "✅ TimeSpan installed successfully!"
echo

# Check if user wants to set up the 'ts' command
echo "🔧 Would you like to set up the 'ts' magic command for Git integration? (y/N)"
read -r response

if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
    echo "📥 Setting up 'ts' command integration..."
    
    # Download integration script
    curl -fsSL https://raw.githubusercontent.com/hisgarden/TimeSpan/main/.timespan_warp_integration.zsh -o ~/.timespan_warp_integration.zsh
    
    # Detect shell and config file
    if [[ "$SHELL" == */zsh ]]; then
        CONFIG_FILE="$HOME/.zshrc"
    elif [[ "$SHELL" == */bash ]]; then
        CONFIG_FILE="$HOME/.bashrc"
    else
        CONFIG_FILE="$HOME/.profile"
    fi
    
    # Check if already configured
    if ! grep -q "timespan_warp_integration" "$CONFIG_FILE" 2>/dev/null; then
        echo "# TimeSpan Warp Integration" >> "$CONFIG_FILE"
        echo "source ~/.timespan_warp_integration.zsh" >> "$CONFIG_FILE"
        echo "✅ Added TimeSpan integration to $CONFIG_FILE"
    else
        echo "ℹ️  TimeSpan integration already configured in $CONFIG_FILE"
    fi
    
    echo
    echo "🎉 Setup complete! Restart your terminal or run:"
    echo "   source $CONFIG_FILE"
    echo
    echo "💡 Now you can use 'ts <message>' in any Git repository to commit + track time!"
else
    echo "ℹ️  Skipped 'ts' command setup. You can set it up later if needed."
fi

echo
echo "🏁 Installation Summary:"
echo "   ✅ TimeSpan CLI installed"
echo "   📖 Usage: timespan --help"
echo "   🌐 Documentation: https://github.com/hisgarden/TimeSpan"
echo
echo "🚀 Quick start:"
echo "   timespan project create \"My Project\""
echo "   timespan start \"My Project\" --task \"Getting started\""
echo "   timespan status"
echo
echo "Happy time tracking! ⏱️"
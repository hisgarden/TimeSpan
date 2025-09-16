class Timespan < Formula
  desc "Local time tracking application built with Rust"
  homepage "https://github.com/hisgarden/TimeSpan"
  url "https://github.com/hisgarden/TimeSpan/archive/refs/tags/v1.1.0.tar.gz"
  sha256 "963b3e756facd1389509ab4b05b9c6cc69c4346f72652c4afc5ad493ef48c6ac"
  license "MIT"

  # Use system Rust if available, otherwise install minimal Rust
  depends_on "rust" => :build

  def install
    # Build with optimizations
    system "cargo", "install", 
           "--path", ".",
           "--root", prefix
  end

  test do
    # Test that the binary was installed and can run
    assert_match "A local time tracking application", shell_output("#{bin}/timespan --help")

    # Test basic functionality with temporary database
    system "#{bin}/timespan", "--database", "#{testpath}/test.db", "project", "create", "Test Project"
    assert_match "Test Project", shell_output("#{bin}/timespan --database #{testpath}/test.db project list")

    # Test status (should show no active timer)
    assert_match "No active timer", shell_output("#{bin}/timespan --database #{testpath}/test.db status")
  end
end

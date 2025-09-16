class Timespan < Formula
  desc "Local time tracking application built with Rust"
  homepage "https://github.com/hisgarden/TimeSpan"
  url "https://github.com/hisgarden/TimeSpan/releases/download/v1.1.0/timespan-v1.1.0-x86_64-apple-darwin.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"

  # No build dependencies needed - just download and install the binary
  def install
    bin.install "timespan"
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


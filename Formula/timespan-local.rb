class TimespanLocal < Formula
  desc "A local time tracking application built with Rust"
  homepage "https://github.com/hisgarden/TimeSpan"
  url "file:///Users/jwen/workspace/ml/TimeSpan"
  version "0.1.0"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--path", ".", "--root", prefix
  end

  test do
    # Test that the binary was installed and can run
    assert_match "A local time tracking application", shell_output("#{bin}/timespan --help")
    
    # Create a temporary database for testing
    testpath = Pathname.new(Dir.mktmpdir)
    cd testpath do
      # Test basic functionality
      system "#{bin}/timespan", "--database", "#{testpath}/test.db", "project", "create", "Test Project"
      assert_match "Test Project", shell_output("#{bin}/timespan --database #{testpath}/test.db project list")
      
      # Test status (should show no active timer)
      assert_match "No active timer", shell_output("#{bin}/timespan --database #{testpath}/test.db status")
    end
  end
end
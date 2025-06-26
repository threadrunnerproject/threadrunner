class Threadrunner < Formula
  desc "Modern system-level runtime for local LLM execution"
  homepage "https://github.com/threadrunnerproject/threadrunner"
  url "https://github.com/threadrunnerproject/threadrunner/releases/download/v0.1.0/threadrunner-macos-universal.tar.gz"
  sha256 "<PLACEHOLDER_SHA>"
  license "MIT"
  version "0.1.0"

  def install
    bin.install "threadrunner"
  end

  test do
    # Test that the binary exists and responds to --help
    assert_match "ThreadRunner", shell_output("#{bin}/threadrunner --help")
  end
end 
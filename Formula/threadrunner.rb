class Threadrunner < Formula
  desc "Modern system-level runtime for local LLM execution"
  homepage "https://github.com/vivienhenz24/threadrunner"
  url "https://github.com/vivienhenz24/threadrunner/releases/download/v0.1.0/threadrunner-macos-universal.tar.gz"
  sha256 "344d736363ea0548c35651288139da7dabbdcb79e2213316cddd961c148f5727"
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
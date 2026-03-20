class Sxmc < Formula
  desc "AI-agnostic Skills x MCP x CLI pipeline"
  homepage "https://github.com/aihxp/sxmc"
  url "https://github.com/aihxp/sxmc/archive/refs/tags/v0.1.2.tar.gz"
  sha256 "4c6f5802255801d6a20da391dab5de2613d5031756e7781769802f6fdf56510f"
  license "MIT"
  head "https://github.com/aihxp/sxmc.git", branch: "master"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    assert_match "sxmc", shell_output("#{bin}/sxmc --version")
  end
end

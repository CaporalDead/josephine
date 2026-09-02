class Josephine < Formula
  desc "Your computer's guardian angel — a local Linux system watcher"
  homepage "https://github.com/systm-d/josephine"
  url "https://github.com/systm-d/josephine/archive/refs/tags/v0.13.0.tar.gz"
  sha256 "6a5332738d4b0e02774abba0e1c5925683a68a4ee36134bba8a7948ea851eeef"
  license "MIT OR Apache-2.0"
  head "https://github.com/systm-d/josephine.git", branch: "main"

  depends_on "rust" => :build
  # Joséphine is Linux-native (systemd, /sys/class/thermal, libnotify).
  depends_on :linux

  def install
    # Root Cargo.toml is a virtual workspace, so point cargo at the binary crate.
    system "cargo", "install", *std_cargo_args(path: "crates/josephine")
  end

  test do
    system bin/"josephine", "--version"
  end
end

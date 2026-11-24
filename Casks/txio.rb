cask "txio" do
  arch arm: "aarch64", intel: "x64"

  version "1.0.3"
  sha256 arm:   "9eaaff93c473b57b4f7d862c01d1290de9ebc5df4b28c89f8b2eeab7c4bb1041",
         intel: "7c0f43645147abb171f1ff9b07a4178fa9e6022652adeca4e4c8edc6bb3fec45"

  url "https://github.com/Txio-labs/txio-desktop/releases/download/v#{version}/Txio_#{version}_#{arch}.dmg"
  name "Txio"
  desc "Multi-chain RPC and transaction console"
  homepage "https://txio-labs.github.io/txio-desktop/"

  livecheck do
    url :url
    strategy :github_latest
  end

  auto_updates true
  depends_on macos: ">= :big_sur"

  app "Txio.app"

  zap trash: [
    "~/Library/Application Support/xyz.txio.desktop",
    "~/Library/Caches/xyz.txio.desktop",
    "~/Library/WebKit/xyz.txio.desktop",
  ]

  caveats <<~EOS
    Txio is not code-signed yet. If macOS says it cannot be opened, run:
      xattr -dr com.apple.quarantine "#{appdir}/Txio.app"
  EOS
end

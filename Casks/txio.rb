cask "txio" do
  arch arm: "aarch64", intel: "x64"

  version "1.0.0"
  sha256 arm:   "26b7d3e51d3be131a2c811514dd87121b0d3e902e9cf006a776216d8c2048f41",
         intel: "dd5338549db287eba4071495e1b9eac2c95416fe287d752e6cb35ae710535b7a"

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

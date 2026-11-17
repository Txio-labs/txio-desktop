cask "txio" do
  arch arm: "aarch64", intel: "x64"

  version "1.0.1"
  sha256 arm:   "7c3961548773ee66f87b8e37a409cfac56e359ca69d578cbbd1e02af6c40a9f2",
         intel: "2962c1eb30f3ee7d3667e9feecf5bc2568ca5d63d8d60d15e185a211d66adc28"

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

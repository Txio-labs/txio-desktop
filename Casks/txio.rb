cask "txio" do
  arch arm: "aarch64", intel: "x64"

  version "1.0.2"
  sha256 arm:   "946db87778ecee6874cda1aea063fccfeaeecf54ce11abd3a95e8b57c4813bd5",
         intel: "41ff07f8328f6e1887963649d67f2eb9041d386d9440f1a0a976aca36e006ad4"

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

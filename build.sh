#!/usr/bin/env bash
set -euo pipefail

readonly TARGET_DIR="/usr/lib/ark"
readonly BIN_DIR="/usr/local/bin"
readonly PMS=(pacman apt-get apt dnf yum zypper apk emerge xbps-install slackpkg opkg)

echo "==> building carrier (release)..."
cargo build --release

echo "==> installing binary to $TARGET_DIR/carrier"
sudo mkdir -p "$TARGET_DIR"
sudo cp target/release/carrier "$TARGET_DIR/carrier"
sudo chmod 755 "$TARGET_DIR/carrier"

echo "==> creating symlinks in $BIN_DIR"
for pm in "${PMS[@]}"; do
    sudo ln -sf "$TARGET_DIR/carrier" "$BIN_DIR/$pm"
done

echo "==> done"
echo "    binary: $TARGET_DIR/carrier"
echo "    symlinks: ${PMS[*]}"

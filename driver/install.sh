#!/usr/bin/env bash
set -euo pipefail

bundle=target/Micless.driver

cargo build --release

rm -rf "$bundle"
mkdir -p "$bundle/Contents/MacOS"
cp target/release/libmicless_driver.dylib "$bundle/Contents/MacOS/Micless"
cp Info.plist "$bundle/Contents/Info.plist"
codesign --force --sign - "$bundle"

sudo rm -rf /Library/Audio/Plug-Ins/HAL/Micless.driver
sudo cp -R "$bundle" /Library/Audio/Plug-Ins/HAL/
sudo killall coreaudiod

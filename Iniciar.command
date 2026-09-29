#!/bin/zsh
# Compila y abre la ventana. Mantener este archivo dentro de la carpeta del proyecto.
set -e
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release --locked
bundle="target/Millennium Falcon.app"
mkdir -p "$bundle/Contents/MacOS"
cp target/release/falcon_diorama "$bundle/Contents/MacOS/falcon_diorama.new"
mv -f "$bundle/Contents/MacOS/falcon_diorama.new" "$bundle/Contents/MacOS/falcon_diorama"
cp platform/macos/Info.plist "$bundle/Contents/Info.plist"
open "$bundle"

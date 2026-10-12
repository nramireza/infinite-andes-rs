#!/usr/bin/env bash
# Compila el frontend WASM y copia el módulo junto al reproductor web.
# Requiere el target wasm32-unknown-unknown (rustup target add wasm32-unknown-unknown).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

CARGO="${CARGO:-cargo}"
"$CARGO" build --release --target wasm32-unknown-unknown -p andes-web

cp target/wasm32-unknown-unknown/release/andes_web.wasm web/andes_web.wasm
echo "web/andes_web.wasm actualizado"
echo "sirve la carpeta web/ y abre index.html, p. ej.:"
echo "  python3 -m http.server 8000 --directory web"

#!/usr/bin/env bash
# Arnés de paridad de píxeles: renderiza la misma escena en JS (Canvas vía Chrome
# headless) y en Rust, y compara píxel a píxel.
#
# Uso: scripts/pixel-check.sh [seed] [hour] [weather] [biome] [moment] [bloom] [season] [aspect] [x]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DOCS="$(cd "$ROOT/.." && pwd)"

SEED="${1:-andes}"
HOUR="${2:-12}"
WEATHER="${3:-clear}"
BIOME="${4:-centro}"
MOMENT="${5:-none}"
BLOOM="${6:-off}"
SEASON="${7:-verano}"
ASPECT="${8:-16:9}"
X="${9:-0}"

W="$(python3 - "$ASPECT" <<'PY'
import sys
s = sys.argv[1].strip()
r = None
for sep in (":", "x", "X", "/"):
    if sep in s:
        a, b = s.split(sep)
        r = float(a) / float(b)
        break
if r is None:
    r = float(s)
print(round(270 * r))
PY
)"
H=270

cd "$ROOT"
cargo build -q -p andes-native --bin pixelcheck

PORT=8137
python3 -m http.server "$PORT" --directory "$DOCS" >/tmp/ia-web.log 2>&1 &
SRV=$!
trap 'kill $SRV 2>/dev/null || true' EXIT
sleep 1.2

URL="http://localhost:$PORT/infinite-andes-rs/scripts/pixel-harness/capture.html?seed=${SEED}&hour=${HOUR}&weather=${WEATHER}&biome=${BIOME}&moment=${MOMENT}&bloom=${BLOOM}&season=${SEASON}&aspect=${ASPECT}&x=${X}"

google-chrome-stable --headless=new --disable-gpu --no-sandbox --hide-scrollbars \
  --force-device-scale-factor=1 --window-size="$W,$H" --virtual-time-budget=4000 \
  --screenshot=/tmp/ia-js.png "$URL" >/tmp/ia-chrome.log 2>&1 || true

./target/debug/pixelcheck /tmp/ia-js.png "$SEED" "$HOUR" "$WEATHER" "$BIOME" "$MOMENT" "$BLOOM" "$SEASON" "$ASPECT" "$X"

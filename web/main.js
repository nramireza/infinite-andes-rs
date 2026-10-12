// Reproductor WASM de Infinite Andes (Rust). Sin bundler: instancia el módulo,
// coloca el framebuffer en un <canvas> y corre el bucle con requestAnimationFrame.
// Parámetros por URL: seed, x, hour, weather, aspect, moment, season, biome,
// bloom, clock (real|fast), fit (cover|contain), fps.

const H = 270;

function hashString(str) {
  let h = 2166136261 >>> 0;
  for (let i = 0; i < str.length; i++) {
    h ^= str.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

function seedToInt(seed) {
  if (typeof seed === "number") return seed >>> 0;
  const s = String(seed ?? "").trim();
  if (s === "") return 0;
  const n = Number(s);
  if (Number.isFinite(n)) return n >>> 0;
  return hashString(s);
}

function ratioFromString(str) {
  if (str == null) return null;
  const t = String(str).trim();
  const m = t.match(/^(\d+(?:\.\d+)?)\s*[:xX/]\s*(\d+(?:\.\d+)?)$/);
  if (m) {
    const a = parseFloat(m[1]);
    const b = parseFloat(m[2]);
    return a > 0 && b > 0 ? a / b : null;
  }
  const n = parseFloat(t);
  return Number.isFinite(n) && n > 0 ? n : null;
}

function writeStr(ex, str) {
  const bytes = new TextEncoder().encode(str);
  const ptr = ex.ia_scratch();
  const view = new Uint8Array(ex.memory.buffer, ptr, Math.min(bytes.length, ex.ia_scratch_len()));
  view.set(bytes.subarray(0, view.length));
  return bytes.length;
}

async function main() {
  const params = new URLSearchParams(location.search);
  const canvas = document.getElementById("scene");
  const ctx = canvas.getContext("2d", { alpha: false });
  ctx.imageSmoothingEnabled = false;

  const resp = await fetch("./andes_web.wasm");
  const { instance } = await WebAssembly.instantiateStreaming(resp, {});
  const ex = instance.exports;

  const ratio = ratioFromString(params.get("aspect")) || 32 / 9;
  const W = Math.max(1, Math.round(H * ratio));
  canvas.width = W;
  canvas.height = H;

  let scene = ex.ia_new(seedToInt(params.get("seed") || "andes"), W, H);

  const setters = ["weather", "biome", "moment", "season", "bloom"];
  for (const key of setters) {
    const v = params.get(key);
    if (v) {
      const len = writeStr(ex, v);
      ex["ia_set_" + key](scene, ex.ia_scratch(), len);
    }
  }

  const clock = params.get("clock") || "real";
  ex.ia_set_clock_fast(scene, clock === "fast" ? 1 : 0);
  const hourParam = params.get("hour");
  if (hourParam != null) ex.ia_set_hour(scene, parseFloat(hourParam));
  ex.ia_set_autoscroll(scene, params.get("x") != null ? 0 : 1);

  // Ajuste a la ventana (cover/contain), escalado por CSS.
  const fit = (params.get("fit") || "contain").toLowerCase() === "cover" ? "cover" : "contain";
  function resize() {
    const availW = window.innerWidth;
    const availH = window.innerHeight;
    const s = fit === "cover" ? Math.max(availW / W, availH / H) : Math.min(availW / W, availH / H);
    canvas.style.width = Math.round(W * s) + "px";
    canvas.style.height = Math.round(H * s) + "px";
  }
  resize();
  window.addEventListener("resize", resize);

  const img = ctx.createImageData(W, H);

  let last = performance.now();
  function frame(now) {
    requestAnimationFrame(frame);
    const dt = Math.min((now - last) / 1000, 0.1);
    last = now;
    if (clock === "real") {
      const d = new Date();
      ex.ia_set_now(scene, d.getFullYear(), d.getMonth(), d.getDate(), d.getHours(), d.getMinutes(), d.getSeconds(), d.getMilliseconds(), -d.getTimezoneOffset() / 60);
    }
    ex.ia_update(scene, dt);
    ex.ia_render(scene);
    const ptr = ex.ia_pixels(scene);
    const len = ex.ia_pixel_len(scene);
    img.data.set(new Uint8ClampedArray(ex.memory.buffer, ptr, len));
    ctx.putImageData(img, 0, 0);
  }
  requestAnimationFrame(frame);
}

main().catch((e) => {
  const el = document.createElement("pre");
  el.id = "err";
  el.textContent = "Infinite Andes · error al cargar el WASM\n" + (e && e.stack ? e.stack : e);
  document.body.appendChild(el);
});

// Açılış penceresi: Rust'tan gelen GERÇEK ilerlemeyi gösterir. React kullanılmaz (hızlı açılış).
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import logoUrl from "../../assets/brand/logo-1024.png";
import { ringGeometry, stepTowards } from "./progress";
import "./splash.css";

interface ProgressEvent {
  percent: number;
  message: string;
}

interface StartupStatus extends ProgressEvent {
  error: string | null;
  done: boolean;
}

/** Açılış ekranı en az bu kadar görünür (yanıp sönmesin). */
const MIN_VISIBLE_MS = 900;
let shownAt = 0;

function $(id: string): HTMLElement {
  const el = document.getElementById(id);
  if (!el) throw new Error(`#${id} yok`);
  return el;
}
const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
const canvas = $("ring") as HTMLCanvasElement;
const ctx = canvas.getContext("2d");
const percentEl = $("percent");
const statusEl = $("status");
($("logo") as HTMLImageElement).src = logoUrl;

let target = 0;
let shown = 0;
let spin = 0;
let currentMessage = "";

function draw() {
  if (!ctx) return;
  const dpr = window.devicePixelRatio || 1;
  const size = 96;
  if (canvas.width !== size * dpr) {
    canvas.width = size * dpr;
    canvas.height = size * dpr;
    canvas.style.width = `${String(size)}px`;
    canvas.style.height = `${String(size)}px`;
  }
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, size, size);
  const g = ringGeometry(size, 7, shown);
  ctx.lineCap = "round";
  ctx.lineWidth = g.stroke;
  // İz
  ctx.strokeStyle = "rgba(255,255,255,0.08)";
  ctx.beginPath();
  ctx.arc(g.c, g.c, g.r, 0, Math.PI * 2);
  ctx.stroke();
  // Gerçek ilerleme: beyazdan maviye degrade
  if (shown > 0.3) {
    const grad = ctx.createConicGradient(g.start, g.c, g.c);
    grad.addColorStop(0, "#ededee");
    grad.addColorStop(Math.max(0.01, shown / 100), "#4f8ff7");
    grad.addColorStop(1, "#4f8ff7");
    ctx.strokeStyle = grad;
    ctx.beginPath();
    ctx.arc(g.c, g.c, g.r, g.start, g.end);
    ctx.stroke();
  }
  // Dönen ışık: sürekli hareket (çalıştığını gösterir), ilerlemeden bağımsız
  if (!reduced && shown < 100) {
    const glow = ctx.createConicGradient(spin, g.c, g.c);
    glow.addColorStop(0, "rgba(255,255,255,0.55)");
    glow.addColorStop(0.12, "rgba(255,255,255,0)");
    glow.addColorStop(1, "rgba(255,255,255,0)");
    ctx.strokeStyle = glow;
    ctx.beginPath();
    ctx.arc(g.c, g.c, g.r, 0, Math.PI * 2);
    ctx.stroke();
  }
}

function frame() {
  shown = reduced ? target : stepTowards(shown, target);
  spin = (spin + 0.07) % (Math.PI * 2);
  const value = Math.round(shown);
  percentEl.textContent = `%${String(value)}`;
  percentEl.setAttribute("aria-valuenow", String(value));
  draw();
  requestAnimationFrame(frame);
}

/** Metin değişince eski satır yukarı kayıp solar, yenisi aşağıdan gelir. */
function setStatus(message: string) {
  if (message === currentMessage) return;
  currentMessage = message;
  const parent = statusEl.parentElement;
  if (!parent || reduced) {
    statusEl.textContent = message;
    return;
  }
  const next = statusEl.cloneNode() as HTMLElement;
  next.removeAttribute("id");
  next.textContent = message;
  next.classList.add("is-entering");
  parent.querySelectorAll(".status__line:not(#status)").forEach((n) => {
    n.remove();
  });
  statusEl.classList.add("is-leaving");
  parent.appendChild(next);
  window.setTimeout(() => {
    statusEl.textContent = message;
    statusEl.classList.remove("is-leaving");
    next.remove();
  }, 320);
}

function showError(message: string) {
  $("progress-view").hidden = true;
  $("error-view").hidden = false;
  $("error-message").textContent = message;
}

$("retry").addEventListener("click", () => {
  $("error-view").hidden = true;
  $("progress-view").hidden = false;
  target = 0;
  shown = 0;
  setStatus("Yeniden deneniyor...");
  void invoke("startup_retry");
});
$("open-log").addEventListener("click", () => {
  void invoke("open_log_dir");
});

void listen<ProgressEvent>("startup://progress", (e) => {
  target = Math.max(target, e.payload.percent); // ilerleme asla geri gitmez
  setStatus(e.payload.message);
});
void listen<{ message: string }>("startup://error", (e) => {
  showError(e.payload.message);
});
/** Sayaç %100'e ulaşıp en kısa görünme süresi dolunca solarak kapanır. */
function leaveWhenReady() {
  target = 100;
  const tick = () => {
    if (shown < 100 || performance.now() - shownAt < MIN_VISIBLE_MS) {
      window.setTimeout(tick, 40);
      return;
    }
    document.body.classList.add("is-leaving");
    window.setTimeout(() => void getCurrentWindow().close(), reduced ? 0 : 280);
  };
  tick();
}

void listen("startup://done", () => {
  // Ana pencere ilk karesini boyadı: açılış penceresi solarak kapanır (arada boş ekran yok).
  leaveWhenReady();
});

requestAnimationFrame(frame);
// İçerik boyandıktan sonra görünür ol (beyaz flaş yok).
requestAnimationFrame(() => {
  requestAnimationFrame(() => {
    void getCurrentWindow()
      .show()
      .then(async () => {
        shownAt = performance.now();
        // Görünür olduktan sonra kaçırılmış olabilecek son durumu oku ve hazır olduğunu bildir.
        const s = await invoke<StartupStatus>("splash_ready");
        if (s.error) showError(s.error);
        else {
          target = Math.max(target, s.percent);
          if (s.message) setStatus(s.message);
        }
      });
  });
});

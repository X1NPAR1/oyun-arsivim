import { useEffect, useRef } from "react";
import { loadLevel } from "../../ipc/perf";
import { useMotionLevel } from "../../stores/ui";
import { CHART_THEME } from "./theme";

interface GaugeProps {
  label: string;
  /** 0–100; ölçülemiyorsa `null`. */
  percent: number | null;
  detail: string;
  size?: number;
}

const LEVEL_COLORS = { ok: CHART_THEME.active, high: "#e0a63a", critical: "#e5534b" } as const;

/** Canvas halka gösterge (SVG yok). Değer yumuşakça hedefe ilerler; seviye metinle de belirtilir. */
export function Gauge({ label, percent, detail, size = 132 }: GaugeProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const shown = useRef(0);
  const motion = useMotionLevel();
  const level = percent === null ? null : loadLevel(percent);

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = size * dpr;
    canvas.height = size * dpr;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const target = percent ?? 0;
    const color = level ? LEVEL_COLORS[level.level] : CHART_THEME.text3;
    const stroke = 11;
    const r = size / 2 - stroke;
    const start = Math.PI * 0.75;
    const sweep = Math.PI * 1.5;
    let frame = 0;

    const draw = () => {
      ctx.clearRect(0, 0, size, size);
      ctx.lineCap = "round";
      ctx.lineWidth = stroke;
      ctx.strokeStyle = "rgba(255,255,255,0.07)";
      ctx.beginPath();
      ctx.arc(size / 2, size / 2, r, start, start + sweep);
      ctx.stroke();
      if (shown.current > 0.2) {
        ctx.strokeStyle = color;
        ctx.beginPath();
        ctx.arc(size / 2, size / 2, r, start, start + (sweep * shown.current) / 100);
        ctx.stroke();
      }
    };
    const step = () => {
      const diff = target - shown.current;
      shown.current =
        motion === "off" || Math.abs(diff) < 0.3 ? target : shown.current + diff * 0.18;
      draw();
      if (shown.current !== target) frame = requestAnimationFrame(step);
    };
    step();
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [percent, size, level, motion]);

  return (
    <div
      className="gauge glass"
      role="meter"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={percent === null ? undefined : Math.round(percent)}
    >
      <div className="gauge__ring" style={{ width: size, height: size }}>
        <canvas ref={canvasRef} style={{ width: size, height: size }} />
        <div className="gauge__center">
          <span className="gauge__value">
            {percent === null ? "—" : `%${String(Math.round(percent))}`}
          </span>
          {level && (
            <span className={`gauge__level gauge__level--${level.level}`}>{level.label}</span>
          )}
        </div>
      </div>
      <div className="gauge__text">
        <span className="gauge__label">{label}</span>
        <span className="gauge__detail">{detail}</span>
      </div>
    </div>
  );
}

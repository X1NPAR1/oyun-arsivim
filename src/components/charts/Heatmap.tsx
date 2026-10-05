import { useEffect, useRef, useState, type PointerEvent } from "react";
import { formatDuration, WEEKDAYS, WEEKDAYS_SHORT } from "../../stats/aggregate";
import { CHART_THEME, heatColor } from "./theme";

interface HeatmapProps {
  /** [hafta günü (Pzt=0)][saat] saniye */
  data: number[][];
}

const LABEL_W = 44;
const HEADER_H = 22;
const GAP = 3;
const CELL_H = 22;

/** Hafta günü × saat ısı haritası; canvas'a çizilir, hücre üzerinde ipucu gösterir. */
export function Heatmap({ data }: HeatmapProps) {
  const wrapRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [width, setWidth] = useState(0);
  const [hover, setHover] = useState<{ d: number; h: number; x: number; y: number } | null>(null);
  const max = Math.max(1, ...data.flat());
  const height = HEADER_H + 7 * (CELL_H + GAP);
  const cellW = width > 0 ? (width - LABEL_W - 23 * GAP) / 24 : 0;

  useEffect(() => {
    const el = wrapRef.current;
    if (!el) return;
    const ro = new ResizeObserver(([entry]) => {
      if (entry) setWidth(Math.floor(entry.contentRect.width));
    });
    ro.observe(el);
    return () => {
      ro.disconnect();
    };
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx || width === 0) return;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, width, height);
    ctx.font = `600 11px ${CHART_THEME.font}`;
    ctx.fillStyle = CHART_THEME.text3;
    ctx.textBaseline = "middle";
    for (let h = 0; h < 24; h += 3) {
      ctx.textAlign = "center";
      ctx.fillText(
        String(h).padStart(2, "0"),
        LABEL_W + h * (cellW + GAP) + cellW / 2,
        HEADER_H / 2,
      );
    }
    data.forEach((row, d) => {
      const y = HEADER_H + d * (CELL_H + GAP);
      ctx.textAlign = "left";
      ctx.fillStyle = CHART_THEME.text3;
      ctx.fillText(WEEKDAYS_SHORT[d] ?? "", 0, y + CELL_H / 2);
      row.forEach((v, h) => {
        const x = LABEL_W + h * (cellW + GAP);
        ctx.fillStyle = heatColor(v / max);
        ctx.beginPath();
        ctx.roundRect(x, y, cellW, CELL_H, 4);
        ctx.fill();
        if (hover && hover.d === d && hover.h === h) {
          ctx.strokeStyle = CHART_THEME.text1;
          ctx.lineWidth = 1.5;
          ctx.stroke();
        }
      });
    });
  }, [data, width, height, cellW, max, hover]);

  const onMove = (e: PointerEvent<HTMLCanvasElement>) => {
    const r = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - r.left;
    const y = e.clientY - r.top;
    const h = Math.floor((x - LABEL_W) / (cellW + GAP));
    const d = Math.floor((y - HEADER_H) / (CELL_H + GAP));
    if (h >= 0 && h < 24 && d >= 0 && d < 7) setHover({ d, h, x, y });
    else setHover(null);
  };

  const value = hover ? (data[hover.d]?.[hover.h] ?? 0) : 0;

  return (
    <div ref={wrapRef} className="heatmap" style={{ height }}>
      <canvas
        ref={canvasRef}
        style={{ width, height }}
        role="img"
        aria-label="Haftanın günleri ve günün saatlerine göre oynama yoğunluğu"
        onPointerMove={onMove}
        onPointerLeave={() => {
          setHover(null);
        }}
      />
      {hover && (
        <div
          className="chart-tooltip"
          style={{ left: Math.min(hover.x + 12, width - 180), top: hover.y + 14 }}
        >
          <strong>
            {WEEKDAYS[hover.d]} {String(hover.h).padStart(2, "0")}:00–
            {String((hover.h + 1) % 24).padStart(2, "0")}:00
          </strong>
          <span>{formatDuration(value)}</span>
        </div>
      )}
      <div className="heatmap__legend" aria-hidden>
        <span>Az</span>
        {[0.05, 0.25, 0.5, 0.75, 1].map((t) => (
          <i key={t} style={{ background: heatColor(t) }} />
        ))}
        <span>Çok</span>
      </div>
    </div>
  );
}

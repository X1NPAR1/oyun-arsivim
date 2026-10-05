import {
  BarController,
  BarElement,
  CategoryScale,
  Chart,
  Legend,
  LinearScale,
  Tooltip,
  type ChartConfiguration,
} from "chart.js";
import { useEffect, useRef } from "react";
import { useMotionLevel } from "../../stores/ui";
import { formatAxis, niceStep } from "../../stats/aggregate";
import { CHART_THEME } from "./theme";

// Yalnız kullanılan parçalar kaydedilir (paket boyutu).
Chart.register(BarController, BarElement, CategoryScale, LinearScale, Tooltip, Legend);

export interface BarSeries {
  label: string;
  data: number[];
  color: string;
}

interface BarChartProps {
  labels: string[];
  series: BarSeries[];
  /** Eksen ve ipucu değer biçimi (ör. süre). */
  format: (value: number) => string;
  stacked?: boolean;
  horizontal?: boolean;
  height?: number;
  /** Erişilebilir özet (canvas ekran okuyucuya görünmez). */
  ariaLabel: string;
  /** İpucu başlığı: etiket yerine tam metin (ör. tam tarih). */
  titles?: string[];
}

/** Canvas tabanlı çubuk grafik (SVG yok). Tema token'lardan; animasyon seviyesine uyar. */
export function BarChart({
  labels,
  series,
  format,
  stacked = false,
  horizontal = false,
  height = 240,
  ariaLabel,
  titles,
}: BarChartProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const chartRef = useRef<Chart<"bar"> | null>(null);
  const level = useMotionLevel();

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const valueAxis = horizontal ? "x" : "y";
    // Süre ekseni: yuvarlak adımlar (1/5/15 dk, 1/2 sa…) → etiket tekrarı olmaz.
    const maxValue = stacked
      ? Math.max(0, ...labels.map((_, i) => series.reduce((sum, s) => sum + (s.data[i] ?? 0), 0)))
      : Math.max(0, ...series.flatMap((s) => s.data));
    const step = niceStep(maxValue);
    const categoryAxis = horizontal ? "y" : "x";
    const config: ChartConfiguration<"bar"> = {
      type: "bar",
      data: {
        labels,
        datasets: series.map((s) => ({
          label: s.label,
          data: s.data,
          backgroundColor: s.color,
          hoverBackgroundColor: s.color,
          // 2 px yüzey boşluğu: yığılı parçalar ve komşu çubuklar birbirine yapışmaz.
          borderColor: CHART_THEME.surface,
          borderWidth: stacked ? { top: 2 } : 0,
          borderRadius: 4,
          borderSkipped: "start",
          maxBarThickness: horizontal ? 18 : 28,
          categoryPercentage: 0.72,
          barPercentage: 0.9,
        })),
      },
      options: {
        indexAxis: horizontal ? "y" : "x",
        responsive: true,
        maintainAspectRatio: false,
        animation: level === "off" ? false : { duration: level === "reduced" ? 150 : 380 },
        interaction: { mode: "index", intersect: false },
        plugins: {
          legend: {
            display: series.length > 1,
            position: "top",
            align: "end",
            labels: {
              color: CHART_THEME.text2,
              font: { family: CHART_THEME.font, size: 12, weight: 600 },
              boxWidth: 10,
              boxHeight: 10,
              useBorderRadius: true,
              borderRadius: 3,
            },
          },
          tooltip: {
            backgroundColor: CHART_THEME.tooltipBg,
            borderColor: CHART_THEME.grid,
            borderWidth: 1,
            titleColor: CHART_THEME.text1,
            bodyColor: CHART_THEME.text2,
            titleFont: { family: CHART_THEME.font, weight: 700 },
            bodyFont: { family: CHART_THEME.font },
            padding: 10,
            cornerRadius: 10,
            boxPadding: 4,
            callbacks: {
              title: (items) => {
                const i = items[0]?.dataIndex ?? 0;
                return titles?.[i] ?? labels[i] ?? "";
              },
              label: (item) => ` ${item.dataset.label ?? ""}: ${format(Number(item.raw))}`,
            },
          },
        },
        scales: {
          [categoryAxis]: {
            stacked,
            grid: { display: false },
            border: { display: false },
            ticks: {
              color: CHART_THEME.text3,
              font: { family: CHART_THEME.font, size: 11 },
              maxRotation: 0,
              autoSkipPadding: 12,
            },
          },
          [valueAxis]: {
            stacked,
            beginAtZero: true,
            grid: { color: CHART_THEME.grid },
            border: { display: false },
            ticks: {
              color: CHART_THEME.text3,
              font: { family: CHART_THEME.font, size: 11 },
              stepSize: step,
              callback: (v) => formatAxis(Number(v)),
            },
          },
        },
      },
    };
    chartRef.current?.destroy();
    chartRef.current = new Chart(canvas, config);
    return () => {
      chartRef.current?.destroy();
      chartRef.current = null;
    };
  }, [labels, series, format, stacked, horizontal, level, titles]);

  return (
    <div className="chart" style={{ height }}>
      <canvas ref={canvasRef} role="img" aria-label={ariaLabel} />
    </div>
  );
}

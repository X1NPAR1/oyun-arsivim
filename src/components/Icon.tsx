import { ICON_CODES, type IconName } from "../design/iconCodes";

interface IconProps {
  name: IconName;
  size?: number;
  filled?: boolean;
  className?: string;
  /** Verilirse ikon ekran okuyucuya bu etiketle duyurulur; verilmezse dekoratiftir. */
  label?: string;
}

/** Phosphor ikon fontu (woff2). SVG kullanılmaz — proje kuralı. */
export function Icon({ name, size = 20, filled = false, className, label }: IconProps) {
  const code = ICON_CODES[name][filled ? "fill" : "regular"];
  return (
    <span
      className={`oa-icon${filled ? " oa-icon--fill" : ""}${className ? ` ${className}` : ""}`}
      style={{ fontSize: size }}
      role={label ? "img" : undefined}
      aria-label={label}
      aria-hidden={label ? undefined : true}
    >
      {String.fromCodePoint(code)}
    </span>
  );
}

import type { IconName } from "../design/iconCodes";
import type { NavDirection } from "../design/motion";

export interface NavItem {
  path: string;
  label: string;
  icon: IconName;
}

/** Sidebar sırası aynı zamanda geçiş yönünü belirler: listede aşağı gitmek "ileri"dir. */
export const NAV_ITEMS: readonly NavItem[] = [
  { path: "/", label: "Ana sayfa", icon: "house" },
  { path: "/kutuphane", label: "Kütüphane", icon: "squares-four" },
  { path: "/istatistikler", label: "İstatistikler", icon: "chart-line-up" },
  { path: "/yedekler", label: "Yedekler", icon: "cloud-arrow-up" },
  { path: "/ayarlar", label: "Ayarlar", icon: "gear-six" },
];

function segments(path: string): string[] {
  return path.split("/").filter(Boolean);
}

function topLevelIndex(path: string): number {
  const first = segments(path)[0];
  const root = first ? `/${first}` : "/";
  return NAV_ITEMS.findIndex((item) => item.path === root);
}

/**
 * Geçiş yönü: daha derine inmek (kütüphane → oyun detayı) ileri, yukarı çıkmak geri;
 * aynı derinlikte sidebar sırasına göre.
 */
export function navDirection(from: string, to: string): NavDirection {
  const fromDepth = segments(from).length;
  const toDepth = segments(to).length;
  const fromTop = topLevelIndex(from);
  const toTop = topLevelIndex(to);
  if (fromTop === toTop && fromDepth !== toDepth) return toDepth > fromDepth ? 1 : -1;
  return toTop >= fromTop ? 1 : -1;
}

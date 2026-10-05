import { motion } from "motion/react";
import { NavLink, useLocation } from "react-router";
import { spring } from "../../design/motion";
import { NAV_ITEMS } from "../../navigation/routes";
import { useMotionLevel } from "../../stores/ui";
import { Icon } from "../Icon";

function isActive(itemPath: string, pathname: string): boolean {
  return itemPath === "/" ? pathname === "/" : pathname.startsWith(itemPath);
}

/** Pencere kenarından boşluklu, yüzen cam şerit. Aktif öğenin ışıması öğeler arasında kayar. */
export function Sidebar() {
  const { pathname } = useLocation();
  const level = useMotionLevel();

  return (
    <nav className="sidebar glass" aria-label="Ana menü">
      <ul className="sidebar__list">
        {NAV_ITEMS.map((item) => {
          const active = isActive(item.path, pathname);
          return (
            <li key={item.path}>
              <NavLink
                to={item.path}
                className={`sidebar__item${active ? " is-active" : ""}`}
                aria-current={active ? "page" : undefined}
                title={item.label}
              >
                {active && (
                  <motion.span
                    layoutId="sidebar-active"
                    className="sidebar__glow"
                    transition={level === "full" ? spring.gentle : { duration: 0 }}
                  />
                )}
                <Icon name={item.icon} size={22} filled={active} className="sidebar__icon" />
                <span className="sidebar__label">{item.label}</span>
              </NavLink>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}

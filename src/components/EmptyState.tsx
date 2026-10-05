import type { ReactNode } from "react";
import type { IconName } from "../design/iconCodes";
import { Icon } from "./Icon";

interface EmptyStateProps {
  icon: IconName;
  title: string;
  description: string;
  children?: ReactNode;
}

export function EmptyState({ icon, title, description, children }: EmptyStateProps) {
  return (
    <section className="empty-state glass">
      <span className="empty-state__halo">
        <Icon name={icon} size={34} filled />
      </span>
      <h2 className="text-h2">{title}</h2>
      <p className="empty-state__text">{description}</p>
      {children}
    </section>
  );
}

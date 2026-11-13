import type { ReactNode } from "react";

interface SectionHeadingProps {
  eyebrow: string;
  title: string;
  subtitle: string;
  action?: ReactNode;
}

export function SectionHeading({
  eyebrow,
  title,
  subtitle,
  action,
}: SectionHeadingProps) {
  return (
    <div className="section-heading">
      <div>
        <p className="eyebrow">{eyebrow}</p>
        <h1>{title}</h1>
        <p className="subtitle">{subtitle}</p>
      </div>

      {action}
    </div>
  );
}
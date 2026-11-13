import type { LucideIcon } from "lucide-react";

import { HISTORY_LENGTH } from "../../hooks/useSystem";
import { Sparkline } from "../../components/system/Sparkline";
import { UsageBar } from "../../components/system/UsageBar";

interface StatCardProps {
  icon: LucideIcon;
  label: string;
  /** Persentase 0-100, atau null saat data belum tersedia. */
  value: number | null;
  detail: string;
  history?: number[];
}

export function StatCard({
  icon: Icon,
  label,
  value,
  detail,
  history,
}: StatCardProps) {
  return (
    <div className="widget stat-card">
      <div className="stat-card-top">
        <span className="widget-label">{label}</span>
        <Icon size={18} />
      </div>

      <p className="stat-value">
        {value === null ? "--" : Math.round(value)}
        <small>%</small>
      </p>

      {history && <Sparkline values={history} length={HISTORY_LENGTH} />}

      <UsageBar value={value ?? 0} label={label} />

      <p className="stat-detail">{detail}</p>
    </div>
  );
}
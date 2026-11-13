interface UsageBarProps {
  /** Persentase 0-100. */
  value: number;
  thin?: boolean;
  label?: string;
}

/** Bar pemakaian. Warna berubah di 75% (warning) dan 90% (critical). */
export function UsageBar({ value, thin = false, label }: UsageBarProps) {
  const clamped = Math.min(100, Math.max(0, value));
  const level =
    clamped >= 90 ? "critical" : clamped >= 75 ? "warning" : "normal";

  return (
    <div
      className={`usage-bar ${thin ? "usage-bar-thin" : ""}`}
      data-level={level}
      role="progressbar"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={Math.round(clamped)}
    >
      <span style={{ width: `${clamped}%` }} />
    </div>
  );
}
import { HISTORY_LENGTH, useSystemStats } from "../../hooks/useSystem";

import { formatBytes, toPercent } from "../../lib/format";
import { getPrimaryDisk } from "../../lib/system";

import { Sparkline } from "../../components/system/Sparkline";
import { UsageBar } from "../../components/system/UsageBar";
import { DesktopWidgetFrame } from "../../components/widget/DesktopWidgetFrame";

interface WidgetRowProps {
  label: string;
  value: number | null;
  detail: string;
  history?: number[];
}

function WidgetRow({ label, value, detail, history }: WidgetRowProps) {
  return (
    <div className="widget-row">
      <div className="widget-row-head">
        <span className="widget-row-label">{label}</span>
        <span className="widget-row-detail">{detail}</span>
        <span className="widget-row-value">
          {value === null ? "--" : Math.round(value)}%
        </span>
      </div>

      {history && <Sparkline values={history} length={HISTORY_LENGTH} />}

      <UsageBar value={value ?? 0} thin label={label} />
    </div>
  );
}

export function SystemDesktopWidget() {
  const { stats, error, cpuHistory, memoryHistory } = useSystemStats(2000);

  const memory = stats?.memory;
  const disk = stats ? getPrimaryDisk(stats.disks) : undefined;

  return (
    <DesktopWidgetFrame title="SYSTEM">
      {error && !stats ? (
        <p className="widget-row-detail">System data unavailable</p>
      ) : (
        <>
          <WidgetRow
            label="CPU"
            value={stats?.cpu.usage ?? null}
            detail={stats ? `${stats.cpu.perCore.length} threads` : ""}
            history={cpuHistory}
          />

          <WidgetRow
            label="RAM"
            value={memory ? toPercent(memory.used, memory.total) : null}
            detail={
              memory
                ? `${formatBytes(memory.used)} / ${formatBytes(memory.total)}`
                : ""
            }
            history={memoryHistory}
          />

          <WidgetRow
            label="DISK"
            value={disk ? toPercent(disk.total - disk.available, disk.total) : null}
            detail={disk ? `${formatBytes(disk.available)} free` : ""}
          />
        </>
      )}
    </DesktopWidgetFrame>
  );
}

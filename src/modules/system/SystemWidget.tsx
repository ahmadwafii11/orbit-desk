import { Activity } from "lucide-react";

import { UsageBar } from "../../components/system/UsageBar";
import { formatBytes, toPercent } from "../../lib/format";
import { useAppStore } from "../../stores/appStores";
import { getPrimaryDisk } from "../../lib/system";
import { useSystemStats } from "../../hooks/useSystem";

interface MiniMeterProps {
  label: string;
  value: number | null;
  detail: string;
}

function MiniMeter({ label, value, detail }: MiniMeterProps) {
  return (
    <div className="mini-meter">
      <div className="mini-meter-head">
        <span className="mini-meter-label">{label}</span>
        <span className="mini-meter-value">
          {value === null ? "--" : Math.round(value)}%
        </span>
      </div>

      <UsageBar value={value ?? 0} thin label={label} />

      <p className="mini-meter-detail">{detail}</p>
    </div>
  );
}

// Ringkasan CPU / RAM / disk untuk Dashboard. Klik untuk membuka modul System.
export function SystemWidget() {
  const onNavigate = useAppStore((s) => s.setActiveModule);
  const { stats, error } = useSystemStats();

  const memory = stats?.memory;
  const disk = stats ? getPrimaryDisk(stats.disks) : undefined;
  const diskUsed = disk ? disk.total - disk.available : 0;

  return (
    <button
      className="widget system-widget"
      onClick={() => onNavigate("system")}
    >
      <div className="widget-top">
        <span className="widget-label">SYSTEM</span>
        <Activity size={17} />
      </div>

      {error && !stats ? (
        <p className="widget-description">System data unavailable</p>
      ) : (
        <div className="system-widget-meters">
          <MiniMeter
            label="CPU"
            value={stats?.cpu.usage ?? null}
            detail={stats ? `${stats.cpu.perCore.length} threads` : "Reading..."}
          />

          <MiniMeter
            label="RAM"
            value={memory ? toPercent(memory.used, memory.total) : null}
            detail={
              memory
                ? `${formatBytes(memory.used)} / ${formatBytes(memory.total)}`
                : "Reading..."
            }
          />

          <MiniMeter
            label="DISK"
            value={disk ? toPercent(diskUsed, disk.total) : null}
            detail={disk ? `${formatBytes(disk.available)} free` : "No disk"}
          />
        </div>
      )}
    </button>
  );
}
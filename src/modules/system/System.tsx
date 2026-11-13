import { ArrowLeftRight, Clock, Cpu, MemoryStick } from "lucide-react";

import { SectionHeading } from "../../components/layout/SectionHeading";
import { UsageBar } from "../../components/system/UsageBar";
import { formatBytes, formatDuration, toPercent } from "../../lib/format";
import { DiskList } from "./DiskList";
import { StatCard } from "./StatCard";
import { useSystemInfo, useSystemStats } from "../../hooks/useSystem";

export function System() {
  const { stats, error, cpuHistory, memoryHistory } = useSystemStats();
  const info = useSystemInfo();

  const memory = stats?.memory;
  const memoryPercent = memory ? toPercent(memory.used, memory.total) : null;
  const swapPercent = memory ? toPercent(memory.swapUsed, memory.swapTotal) : 0;

  return (
    <>
      <SectionHeading
        eyebrow="DEVICE INFORMATION"
        title="System."
        subtitle={
          info ? `${info.osName} ${info.osVersion}`.trim() : "Your device overview."
        }
        action={
          <div className="heading-actions">
            {stats && (
              <div className="uptime-label">
                <Clock size={14} />
                <span>Up {formatDuration(stats.uptimeSecs)}</span>
              </div>
            )}
          </div>
        }
      />

      {error && <div className="widget system-error">{error}</div>}

      <div className="system-grid">
        <StatCard
          icon={Cpu}
          label="CPU"
          value={stats?.cpu.usage ?? null}
          detail={info ? `${info.cpuCores} threads` : "Reading CPU..."}
          history={cpuHistory}
        />

        <StatCard
          icon={MemoryStick}
          label="MEMORY"
          value={memoryPercent}
          detail={
            memory
              ? `${formatBytes(memory.used)} of ${formatBytes(memory.total)}`
              : "Reading memory..."
          }
          history={memoryHistory}
        />
      </div>

      <div className="widget system-section">
        <div className="stat-card-top">
          <span className="widget-label">SWAP</span>
          <ArrowLeftRight size={18} />
        </div>

        {memory && memory.swapTotal === 0 ? (
          <p className="stat-detail">No swap configured.</p>
        ) : (
          <>
            <UsageBar value={swapPercent} thin label="Swap" />
            <p className="stat-detail">
              {memory
                ? `${formatBytes(memory.swapUsed)} of ${formatBytes(memory.swapTotal)} · ${Math.round(swapPercent)}%`
                : "Reading swap..."}
            </p>
          </>
        )}
      </div>

      <DiskList disks={stats?.disks ?? []} />

      {stats && stats.cpu.perCore.length > 0 && (
        <div className="widget system-section">
          <div className="stat-card-top">
            <span className="widget-label">CPU THREADS</span>
            <Cpu size={18} />
          </div>

          <div className="core-grid">
            {stats.cpu.perCore.map((usage, index) => (
              <div
                key={index}
                className="core-bar"
                title={`Thread ${index + 1}: ${Math.round(usage)}%`}
              >
                <span style={{ height: `${usage}%` }} />
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="widget system-section system-info">
        <h3>System information</h3>

        <dl>
          <div>
            <dt>Operating system</dt>
            <dd>{info ? `${info.osName} ${info.osVersion}`.trim() : "--"}</dd>
          </div>
          <div>
            <dt>Kernel</dt>
            <dd>{info?.kernelVersion || "--"}</dd>
          </div>
          <div>
            <dt>Host name</dt>
            <dd>{info?.hostName || "--"}</dd>
          </div>
          <div>
            <dt>Processor</dt>
            <dd>{info?.cpuBrand || "--"}</dd>
          </div>
          <div>
            <dt>Application</dt>
            <dd>Orbit Desk v0.1.0 · Local mode</dd>
          </div>
        </dl>
      </div>
    </>
  );
}

import { HardDrive } from "lucide-react";

import type { DiskStats } from "../../types";
import { UsageBar } from "../../components/system/UsageBar";
import { formatBytes, toPercent } from "../../lib/format";


export function DiskList({ disks }: { disks: DiskStats[] }) {
  return (
    <div className="widget system-section">
      <div className="stat-card-top">
        <span className="widget-label">STORAGE</span>
        <HardDrive size={18} />
      </div>

      {disks.length === 0 ? (
        <p className="stat-detail">No disks detected.</p>
      ) : (
        <ul className="disk-list">
          {disks.map((disk) => {
            const used = disk.total - disk.available;
            const usedPercent = toPercent(used, disk.total);

            return (
              <li key={`${disk.mountPoint}-${disk.name}`}>
                <div className="disk-row-head">
                  <span className="disk-name" title={disk.name || disk.mountPoint}>
                    {disk.mountPoint}
                  </span>
                  <span className="disk-badges">
                    {disk.kind !== "Unknown" && (
                      <span className="disk-badge">{disk.kind}</span>
                    )}
                    {disk.removable && (
                      <span className="disk-badge">Removable</span>
                    )}
                    {disk.fileSystem && (
                      <span className="disk-badge">{disk.fileSystem}</span>
                    )}
                  </span>
                </div>

                <UsageBar
                  value={usedPercent}
                  thin
                  label={`Disk ${disk.mountPoint}`}
                />

                <p className="stat-detail">
                  {formatBytes(disk.available)} free of {formatBytes(disk.total)}{" "}
                  · {Math.round(usedPercent)}% used
                </p>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
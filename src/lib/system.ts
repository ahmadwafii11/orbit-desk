import type { DiskStats } from "../types";

/**
 * Pilih disk utama untuk ringkasan di Dashboard:
 * root "/" (Linux/macOS) atau "C:\" (Windows), selain itu disk terbesar.
 */
export function getPrimaryDisk(disks: DiskStats[]): DiskStats | undefined {
  const root = disks.find(
    (d) => d.mountPoint === "/" || /^C:[\\/]?$/i.test(d.mountPoint),
  );
  if (root) return root;

  return [...disks].sort((a, b) => b.total - a.total)[0];
}
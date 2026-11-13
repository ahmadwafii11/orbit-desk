// Cermin dari struct di src-tauri/src/system.rs (serde camelCase).
// Semua ukuran dalam byte, semua persentase 0-100.

export interface SystemInfo {
  osName: string;
  osVersion: string;
  kernelVersion: string;
  hostName: string;
  cpuBrand: string;
  cpuCores: number;
}

export interface CpuStats {
  usage: number;
  perCore: number[];
}

export interface MemoryStats {
  total: number;
  used: number;
  available: number;
  swapTotal: number;
  swapUsed: number;
}

export interface DiskStats {
  name: string;
  mountPoint: string;
  fileSystem: string;
  kind: "SSD" | "HDD" | "Unknown";
  removable: boolean;
  total: number;
  available: number;
}

export interface SystemStats {
  cpu: CpuStats;
  memory: MemoryStats;
  disks: DiskStats[];
  uptimeSecs: number;
}
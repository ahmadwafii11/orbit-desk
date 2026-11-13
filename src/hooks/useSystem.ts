import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import type { SystemInfo, SystemStats } from "../types";

/** Jumlah titik riwayat yang disimpan untuk grafik sparkline. */
export const HISTORY_LENGTH = 40;

const DEFAULT_INTERVAL_MS = 2000;

function errorMessage(error: unknown): string {
  // Di luar Tauri (misalnya `pnpm dev` di browser biasa) invoke() akan gagal.
  const message = String(error);
  if (message.includes("__TAURI_INTERNALS__") || message.includes("invoke")) {
    return "Tauri runtime tidak tersedia. Jalankan app dengan `pnpm tauri dev`.";
  }
  return message;
}

function pushHistory(history: number[], value: number): number[] {
  return [...history.slice(-(HISTORY_LENGTH - 1)), value];
}

/**
 * Polling `get_system_stats` secara berkala.
 *
 * - Request berikutnya dijadwalkan setelah yang sebelumnya selesai, jadi
 *   tidak pernah menumpuk.
 * - Polling dilewati saat jendela tersembunyi untuk menghemat CPU.
 * - Polling berhenti otomatis saat komponen di-unmount.
 */
export function useSystemStats(intervalMs = DEFAULT_INTERVAL_MS) {
  const [stats, setStats] = useState<SystemStats | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [cpuHistory, setCpuHistory] = useState<number[]>([]);
  const [memoryHistory, setMemoryHistory] = useState<number[]>([]);

  useEffect(() => {
    let cancelled = false;
    let timer: number | undefined;

    const schedule = () => {
      if (!cancelled) timer = window.setTimeout(tick, intervalMs);
    };

    const tick = async () => {
      if (!document.hidden) {
        try {
          const next = await invoke<SystemStats>("get_system_stats");
          if (cancelled) return;

          setStats(next);
          setError(null);
          setCpuHistory((h) => pushHistory(h, next.cpu.usage));
          setMemoryHistory((h) =>
            pushHistory(
              h,
              next.memory.total ? (next.memory.used / next.memory.total) * 100 : 0,
            ),
          );
        } catch (e) {
          if (cancelled) return;
          setError(errorMessage(e));
        }
      }
      schedule();
    };

    tick();

    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [intervalMs]);

  return { stats, error, cpuHistory, memoryHistory };
}

/** Info statis (OS, hostname, CPU). Dibaca sekali saat mount. */
export function useSystemInfo() {
  const [info, setInfo] = useState<SystemInfo | null>(null);

  useEffect(() => {
    let cancelled = false;

    invoke<SystemInfo>("get_system_info")
      .then((value) => {
        if (!cancelled) setInfo(value);
      })
      .catch(() => {
        // Error ditampilkan oleh useSystemStats; info statis cukup dikosongkan.
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return info;
}
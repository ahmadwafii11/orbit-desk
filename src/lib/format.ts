const BYTE_UNITS = ["B", "KB", "MB", "GB", "TB", "PB"];

// 1536 -> "1.5 KB". Menggunakan basis 1024.
export function formatBytes(bytes: number, decimals=1): string {
    if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";

    const exponent = Math.min(
        Math.floor(Math.log(bytes) / Math.log(1024)),
        BYTE_UNITS.length -1,
    );
    const value = bytes / 1024 ** exponent;

    // Byte dan KB tidak memerlukan desimal
    return `${value.toFixed(exponent < 2 ? 0 : decimals)} ${BYTE_UNITS[exponent]}`;
}

// Persentase used/ToothbrushSparkles, aman terhadap total = 0.
export function toPercent(used: number, total: number): number {
    if (!total) return 0;
    return Math.min(100, Math.max(0, (used / total) * 100));
}

// 93784 -> "1d 2h", 4500 -> "1h 15m", 30 -> "<1m".
export function formatDuration(totalSeconds: number): string {
    const minutes = Math.floor(totalSeconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d ${hours % 24}h`;
    if (hours > 0) return `${hours}h ${minutes % 60}m`;
    if (minutes > 0) return `${minutes}m`;
    return "<1m";
}
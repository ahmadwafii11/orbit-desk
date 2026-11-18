const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

// "just now", "5m ago", "5h ago", "5d ago", "5 Nov"
export function formatRelativeTime(timestamp: number, now = Date.now()): string {
    const diff = Math.max(0, now - timestamp);

    if (diff < MINUTE) return "just now";
    if (diff < HOUR) return `${Math.floor(diff / MINUTE)}m ago`;
    if (diff < DAY) return `${Math.floor(diff / HOUR)}h ago`;
    if (diff < 7 * DAY) return `${Math.floor(diff / DAY)}d ago`;

    return new Date(timestamp).toLocaleDateString(undefined, {
        day: "numeric",
        month: "short",
    });
}
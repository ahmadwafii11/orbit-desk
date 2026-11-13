// Membaca daftar disk dan menyaring volume yang bukan penyimpanan sungguhan.
use sysinfo::{
    DiskKind,
    Disks,
};

use crate::models::system::DiskStats;

// File system semu (bukan penyimpanna sungguhan) yang disembunyikan.
const IGNORED_FILE_SYSTEMS: &[&str] = &[
    "tmpfs",
    "devtmpfs",
    "ramfs",
    "squashfs",
    "overlay",
    "efivarfs",
    "proc",
    "sysfs",
    "devfs",
    "autofs",
    "cgroup",
    "cgroup2",
    "nsfs",
    "bpf",
    "tracefs",
    "debugfs",
    "securityfs",
    "configfs",
    "fusectl",
    "pstore",
    "mqueue",
    "hugetlbfs",
    "binfmt_misc",
    "fuse.portal",
    "fuse.gvfsd-fuse",
];

// Mount point yang disembunyikan: snap/docker di Linux, volume internal di macOS.
// Windoes tidak terpengaruh karena path-nya berbentuk `C:\.
const IGNORED_MOUNT_PREFIXES: &[&str] = &[
    "/snap",
    "/boot",
    "/var/lib/docker",
    "/var/lib/containers",
    "/System/Volumes",
    "/private/var",
    "/Library/Developer",
    "/Volumes/com.apple.TimeMachine.localsnapshots",
];

fn is_ignored_mount(mount: &str) -> bool {
    IGNORED_MOUNT_PREFIXES.iter().any(|prefix| {
        mount
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    })
}

pub fn collect_disks(disks: &Disks) -> Vec<DiskStats> {
    let mut result: Vec<DiskStats> = Vec::new();

    for disk in disks.iter() {
        let total = disk.total_space();
        if total == 0 {
            continue;
        }

        let file_system = disk.file_system().to_string_lossy().into_owned();
        if IGNORED_FILE_SYSTEMS.contains(&file_system.to_lowercase().as_str()) {
            continue;
        }

        let mount_point = disk.mount_point().to_string_lossy().into_owned();
        if is_ignored_mount(&mount_point) {
            continue;
        }

        let stats = DiskStats {
            name: disk.name().to_string_lossy().into_owned(),
            mount_point,
            file_system,
            kind: match disk.kind() {
                DiskKind::SSD => "SSD",
                DiskKind::HDD => "HDD",
                _ => "Unknown",
            },
            removable: disk.is_removable(),
            total,
            available: disk.available_space(),
        };

        // Di Linux/macOS satu disk bisa muncul di beberapa mount point (misalnya subvolume btrfs: / dan /home). 
        // Ambil satu saja, yang mount point-nya paling pendek. 
        // Di Windows beberapa drive bisa punya label sama, jadi tidak di-dedupe.
        if !cfg!(windows) && !stats.name.is_empty() {
            if let Some(existing) = result
                .iter_mut()
                .find(|d| d.name == stats.name && d.total == stats.total)
            {
                if stats.mount_point.len() < existing.mount_point.len() {
                    *existing = stats;
                }
                continue;
            }
        }

        result.push(stats);
    }

    result.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));
    result
}
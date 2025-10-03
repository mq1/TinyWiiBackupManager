// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use size::Size;
use std::path::Path;
use sysinfo::Disks;

fn get_disk_usage(path: &Path) -> String {
    let disks = Disks::new_with_refreshed_list();

    disks
        .iter()
        .filter(|disk| path.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
        .map(|disk| {
            let total = disk.total_space();
            let used = total - disk.available_space();

            format!("{}/{}", Size::from_bytes(used), Size::from_bytes(total))
        })
        .unwrap_or_default()
}

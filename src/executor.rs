// SPDX-License-Identifier: GPL-3.0-or-later
// Mohamed Hammad
//
// Trademarks:
//   Steelbore, S3cure, S3cure me, S3cure us
//   These names are trademarks of Mohamed Hammad and may not be used
//   to endorse or promote derivative products without prior permission.

use std::process::Command;
use std::fs::{self, OpenOptions};
use std::io::Write;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionStyle {
    MBR,
    GPT,
}

pub struct InstallationOptions {
    pub device: String, // e.g., "/dev/sdb"
    pub part_style: PartitionStyle,
    #[allow(dead_code)]
    pub secure_boot: bool,
    pub reserve_space_mb: u64,
    pub ventoy_path: String, // Path to the original Ventoy directory
    pub dry_run: bool,
}

const VENTOY_SECTOR_NUM: u64 = 65536;

pub fn install_ventoy(opts: InstallationOptions) -> Result<()> {
    log_info("Starting MJVentoy installation...");
    
    // 1. Basic checks
    if !is_block_device(&opts.device) {
        return Err(anyhow!("{} is not a block device", opts.device));
    }

    // 2. Umount disk
    log_info(&format!("Unmounting partitions on {}...", opts.device));
    if !opts.dry_run {
        umount_disk(&opts.device)?;
    } else {
        log_info("[DRY-RUN] Skipping unmount.");
    }

    // 3. Get disk size
    let sector_num = get_disk_sector_count(&opts.device)?;
    log_info(&format!("Disk sector count: {}", sector_num));
    
    // 4. Calculate offsets
    let (part1_start, part1_end, part2_start, part2_end) = calculate_offsets(sector_num, opts.reserve_space_mb, opts.part_style)?;
    log_info(&format!("Offsets: P1({}-{}) P2({}-{})", part1_start, part1_end, part2_start, part2_end));

    // 5. Create partitions
    log_info("Creating partition table...");
    if !opts.dry_run {
        create_partitions(&opts.device, opts.part_style, part1_start, part1_end, part2_start, part2_end)?;
    } else {
        log_info("[DRY-RUN] Skipping partition table creation.");
    }

    // 6. Format partitions
    let part1 = get_part_name(&opts.device, 1);
    let part2 = get_part_name(&opts.device, 2);
    
    if !opts.dry_run {
        log_info("Waiting for device nodes...");
        std::thread::sleep(std::time::Duration::from_secs(2));

        log_info(&format!("Formatting {} as exFAT...", part1));
        format_partition_1(&part1, &opts.ventoy_path)?;
        
        log_info(&format!("Formatting {} as FAT16...", part2));
        format_partition_2(&part2)?;
    } else {
        log_info(&format!("[DRY-RUN] Skipping formatting of {} and {}.", part1, part2));
    }

    // 7. Write boot images
    log_info("Writing bootloader and images...");
    if !opts.dry_run {
        write_images(&opts.device, &part2, &opts.ventoy_path, part2_start)?;
    } else {
        log_info("[DRY-RUN] Skipping image writing.");
    }

    log_info("Installation successfully completed.");
    Ok(())
}

fn log_info(msg: &str) {
    let _ = append_to_log(&format!("[INFO] {}", msg));
}

fn append_to_log(msg: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("ventoy.log")?;
    writeln!(file, "{} - {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), msg)?;
    Ok(())
}

fn is_block_device(path: &str) -> bool {
    fs::metadata(path).map(|m| {
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            m.file_type().is_block_device()
        }
        #[cfg(not(unix))]
        false
    }).unwrap_or(false)
}

fn get_disk_sector_count(device: &str) -> Result<u64> {
    let dev_name = device.trim_start_matches("/dev/");
    let path = format!("/sys/block/{}/size", dev_name);
    let content = fs::read_to_string(path)?;
    Ok(content.trim().parse()?)
}

fn calculate_offsets(sector_num: u64, reserve_mb: u64, style: PartitionStyle) -> Result<(u64, u64, u64, u64)> {
    let part1_start = 2048;
    let reserve_sectors = reserve_mb * 2048;
    
    let overhead = if style == PartitionStyle::GPT { 34 } else { 0 };
    let part1_end = sector_num - reserve_sectors - VENTOY_SECTOR_NUM - 1 - overhead;
    let part2_start = part1_end + 1;
    
    // 4KB Alignment
    let mod_sector = part2_start % 8;
    let final_part1_end = if mod_sector > 0 { part1_end - mod_sector } else { part1_end };
    let final_part2_start = final_part1_end + 1;
    let part2_end = final_part2_start + VENTOY_SECTOR_NUM - 1;
    
    Ok((part1_start, final_part1_end, final_part2_start, part2_end))
}

fn create_partitions(device: &str, style: PartitionStyle, p1_s: u64, p1_e: u64, p2_s: u64, p2_e: u64) -> Result<()> {
    let label = if style == PartitionStyle::GPT { "gpt" } else { "msdos" };
    
    // Force clear partition table first
    Command::new("dd").args(["if=/dev/zero", &format!("of={}", device), "bs=512", "count=100"]).status()?;

    let status = Command::new("parted")
        .args([
            "-s", device, 
            "mklabel", label,
            "unit", "s",
            "mkpart", "primary", "ntfs", &p1_s.to_string(), &p1_e.to_string(),
            "mkpart", "primary", "fat16", &p2_s.to_string(), &p2_e.to_string(),
        ])
        .status()?;
        
    if !status.success() {
        return Err(anyhow!("parted failed"));
    }
    
    // Set boot flag for MBR
    if style == PartitionStyle::MBR {
        Command::new("parted").args(["-s", device, "set", "1", "boot", "on"]).status()?;
    }
    
    Ok(())
}

fn format_partition_1(part: &str, ventoy_path: &str) -> Result<()> {
    // Detect architecture
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "x86" => "i386",
        "aarch64" => "aarch64",
        "mips64" => "mips64el",
        _ => "i386",
    };
    
    let tool = format!("{}/INSTALL/tool/{}/mkexfatfs", ventoy_path, arch);
    
    if !fs::metadata(&tool).is_ok() {
        return Err(anyhow!("mkexfatfs tool not found at {}", tool));
    }

    let status = Command::new(tool)
        .args(["n", "Ventoy", part])
        .status()?;
    
    if !status.success() {
        return Err(anyhow!("mkexfatfs failed"));
    }
    Ok(())
}

fn format_partition_2(part: &str) -> Result<()> {
    let status = Command::new("mkfs.vfat")
        .args(["-F", "16", "-n", "VTOYEFI", part])
        .status()?;
    if !status.success() {
        return Err(anyhow!("mkfs.vfat failed"));
    }
    Ok(())
}

fn write_images(device: &str, _part: &str, ventoy_path: &str, part2_start: u64) -> Result<()> {
    // Basic writing of boot components
    let boot_img = format!("{}/INSTALL/boot/boot.img", ventoy_path);
    let core_img_xz = format!("{}/INSTALL/boot/core.img.xz", ventoy_path);
    let ventoy_img_xz = format!("{}/INSTALL/ventoy/ventoy.disk.img.xz", ventoy_path);

    // 1. Write boot.img (MBR/GPT protective)
    Command::new("dd")
        .args([&format!("if={}", boot_img), &format!("of={}", device), "bs=1", "count=440", "conv=fsync"])
        .status()?;

    // 2. Write core.img (needs xzcat)
    let core_cmd = format!("xzcat {} | dd of={} bs=512 seek=1 conv=fsync", core_img_xz, device);
    Command::new("sh").arg("-c").arg(core_cmd).status()?;

    // 3. Write ventoy.disk.img to Partition 2
    let ventoy_cmd = format!("xzcat {} | dd of={} bs=512 seek={} conv=fsync", ventoy_img_xz, device, part2_start);
    Command::new("sh").arg("-c").arg(ventoy_cmd).status()?;

    Ok(())
}

fn umount_disk(device: &str) -> Result<()> {
    // Try to unmount all partitions
    for i in 1..9 {
        let part = get_part_name(device, i);
        let _ = Command::new("umount").arg(part).status();
    }
    Ok(())
}

fn get_part_name(device: &str, index: u8) -> String {
    if device.contains("nvme") || device.contains("mmcblk") || device.contains("loop") {
        format!("{}p{}", device, index)
    } else {
        format!("{}{}", device, index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_offsets_mbr_no_reserve() {
        // 16GB disk (31250000 sectors of 512B)
        let sector_num = 31250000;
        let (p1s, _p1e, p2s, p2e) = calculate_offsets(sector_num, 0, PartitionStyle::MBR).unwrap();
        
        assert_eq!(p1s, 2048);
        assert_eq!(p2e, sector_num - 1);
        assert_eq!(p2e - p2s + 1, VENTOY_SECTOR_NUM);
        assert_eq!(p2s % 8, 0); // Aligned
    }

    #[test]
    fn test_calculate_offsets_gpt_with_reserve() {
        let sector_num = 31250000;
        let reserve_mb = 1024;
        let (p1s, _p1e, p2s, p2e) = calculate_offsets(sector_num, reserve_mb, PartitionStyle::GPT).unwrap();
        
        assert_eq!(p1s, 2048);
        assert!(p2e < sector_num - (reserve_mb * 2048));
        assert_eq!(p2e - p2s + 1, VENTOY_SECTOR_NUM);
        assert_eq!(p2s % 8, 0); // Aligned
    }
}

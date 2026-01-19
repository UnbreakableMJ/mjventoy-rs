// SPDX-License-Identifier: GPL-3.0-or-later
//
// MJ Ventoy
// Copyright (c) 2024-2026
// Mohamed Hammad
//
// Trademarks:
//   Steelbore, S3cure, S3cure me, S3cure us, MJ Ventoy
//   These names are trademarks of Mohamed Hammad and may not be used
//   to endorse or promote derivative products without prior permission.

use std::process::Command;
use serde::Deserialize;
use anyhow::{Result, anyhow};

#[derive(Debug, Deserialize)]
pub struct LsblkOutput {
    pub blockdevices: Vec<BlockDevice>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct BlockDevice {

    pub name: String,
    pub model: Option<String>,
    pub size: String,
    pub hotplug: bool,
    #[serde(rename = "type")]
    pub device_type: String,
    pub children: Option<Vec<BlockDevice>>,
}

pub fn get_disks() -> Result<Vec<BlockDevice>> {
    let output = Command::new("lsblk")
        .args(["--json", "--output", "NAME,MODEL,SIZE,HOTPLUG,TYPE"])
        .output()?;
        
    if !output.status.success() {
        return Err(anyhow!("lsblk failed"));
    }
    
    let lsblk: LsblkOutput = serde_json::from_slice(&output.stdout)?;
    
    // Filter for physical disks (not partitions, loops, etc.)
    let disks = lsblk.blockdevices.into_iter()
        .filter(|d| d.device_type == "disk")
        .collect();
        
    Ok(disks)
}

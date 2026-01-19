// SPDX-License-Identifier: GPL-3.0-or-later
// Mohamed Hammad
//
// Trademarks:
//   Steelbore, S3cure, S3cure me, S3cure us
//   These names are trademarks of Mohamed Hammad and may not be used
//   to endorse or promote derivative products without prior permission.

use crate::auth::AuthTool;
use crate::discovery::BlockDevice;
use std::sync::mpsc::Sender;
use anyhow::Result;

pub enum AppMessage {
    InstallationFinished(Result<()>),
    #[allow(dead_code)]
    ProgressUpdate(f32),
}

pub enum AppState {
    AuthSelection,
    DeviceSelection,
    Configuration,
    Installation,
}

pub struct App {
    pub state: AppState,
    pub devices: Vec<BlockDevice>,
    pub selected_device_idx: usize,
    pub auth_tools: Vec<AuthTool>,
    pub selected_auth_idx: usize,
    pub selected_config_idx: usize,
    pub secure_boot: bool,
    pub use_gpt: bool,
    pub reserve_space: u64,
    pub ventoy_path: String,
    pub progress: f32,
    #[allow(dead_code)]
    pub logs: Vec<String>,
    pub tx: Option<Sender<AppMessage>>,
    pub dry_run: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::AuthSelection,
            devices: Vec::new(),
            selected_device_idx: 0,
            auth_tools: Vec::new(),
            selected_auth_idx: 0,
            selected_config_idx: 0,
            secure_boot: true,
            use_gpt: false,
            reserve_space: 0,
            ventoy_path: String::new(),
            progress: 0.0,
            logs: Vec::new(),
            tx: None,
            dry_run: true, // Default to true for safety? Or false? Let's say true for safety.
        }
    }
}


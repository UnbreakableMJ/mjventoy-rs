// SPDX-License-Identifier: GPL-3.0-or-later
// Mohamed Hammad
//
// Trademarks:
//   Steelbore, S3cure, S3cure me, S3cure us
//   These names are trademarks of Mohamed Hammad and may not be used
//   to endorse or promote derivative products without prior permission.

use crossterm::event::{KeyCode, KeyEvent};
use crate::tui::app::{App, AppState, AppMessage};
use crate::executor::{InstallationOptions, PartitionStyle, install_ventoy};
use std::thread;

pub fn handle_key(key: KeyEvent, app: &mut App) {
    match app.state {
        AppState::AuthSelection => handle_auth_input(key, app),
        AppState::DeviceSelection => handle_device_input(key, app),
        AppState::Configuration => handle_config_input(key, app),
        AppState::Installation => {}
    }
}

fn handle_auth_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if app.selected_auth_idx < app.auth_tools.len() - 1 {
                app.selected_auth_idx += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.selected_auth_idx > 0 {
                app.selected_auth_idx -= 1;
            }
        }
        KeyCode::Enter => {
            app.state = AppState::DeviceSelection;
        }
        _ => {}
    }
}

fn handle_device_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if !app.devices.is_empty() && app.selected_device_idx < app.devices.len() - 1 {
                app.selected_device_idx += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.selected_device_idx > 0 {
                app.selected_device_idx -= 1;
            }
        }
        KeyCode::Enter => {
            if !app.devices.is_empty() {
                app.state = AppState::Configuration;
            }
        }
        _ => {}
    }
}

fn handle_config_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if app.selected_config_idx < 4 {
                app.selected_config_idx += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.selected_config_idx > 0 {
                app.selected_config_idx -= 1;
            }
        }
        KeyCode::Enter => {
            match app.selected_config_idx {
                0 => app.secure_boot = !app.secure_boot,
                1 => app.use_gpt = !app.use_gpt,
                2 => {
                    app.reserve_space += 1024;
                }
                3 => app.dry_run = !app.dry_run,
                4 => {
                    app.state = AppState::Installation;
                    let device_name = app.devices[app.selected_device_idx].name.clone();
                    let full_device = if device_name.starts_with('/') { device_name } else { format!("/dev/{}", device_name) };
                    
                    let opts = InstallationOptions {
                        device: full_device,
                        part_style: if app.use_gpt { PartitionStyle::GPT } else { PartitionStyle::MBR },
                        secure_boot: app.secure_boot,
                        reserve_space_mb: app.reserve_space,
                        ventoy_path: app.ventoy_path.clone(),
                        dry_run: app.dry_run,
                    };
                    
                    if let Some(tx) = app.tx.clone() {
                        thread::spawn(move || {
                            let res = install_ventoy(opts);
                            let _ = tx.send(AppMessage::InstallationFinished(res));
                        });
                    }
                }
                _ => {}
            }
        }

        KeyCode::Esc | KeyCode::Backspace => {
            app.state = AppState::DeviceSelection;
        }
        _ => {}
    }
}

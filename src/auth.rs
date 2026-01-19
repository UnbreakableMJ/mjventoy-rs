// SPDX-License-Identifier: GPL-3.0-or-later
// Mohamed Hammad
//
// Trademarks:
//   Steelbore, S3cure, S3cure me, S3cure us
//   These names are trademarks of Mohamed Hammad and may not be used
//   to endorse or promote derivative products without prior permission.

use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthTool {
    Sudo,
    Doas,
    SudoRs,
}

impl AuthTool {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthTool::Sudo => "sudo",
            AuthTool::Doas => "doas",
            AuthTool::SudoRs => "sudo-rs",
        }
    }
}

pub fn is_root() -> bool {
    let uid = unsafe { libc::getuid() };
    uid == 0
}

pub fn get_available_tools() -> Vec<AuthTool> {
    let mut tools = Vec::new();
    
    if Command::new("sudo").arg("-v").status().is_ok() {
        tools.push(AuthTool::Sudo);
    }
    if Command::new("doas").arg("-C").arg("/bin/sh").status().is_ok() {
        tools.push(AuthTool::Doas);
    }
    // sudo-rs might just be 'sudo' in some distros, but if it has its own command:
    if Command::new("sudo-rs").arg("-v").status().is_ok() {
        tools.push(AuthTool::SudoRs);
    }
    
    tools
}

#[allow(dead_code)]
pub fn wrap_command(_tool: AuthTool, _cmd: &mut Command) {

    // This is complex because Command takes args separately.
    // Usually we'd want to prepend the tool and its standard args.
}

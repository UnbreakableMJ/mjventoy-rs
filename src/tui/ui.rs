// SPDX-License-Identifier: GPL-3.0-or-later
// Mohamed Hammad
//
// Trademarks:
//   Steelbore, S3cure, S3cure me, S3cure us
//   These names are trademarks of Mohamed Hammad and may not be used
//   to endorse or promote derivative products without prior permission.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},

    widgets::{Block, Borders, BorderType, List, ListItem, Paragraph, Gauge},
    Frame,
};
use crate::tui::app::{App, AppState};
use crate::theme::SteelboreTheme;

pub fn render(f: &mut Frame, app: &App) {
    let theme = SteelboreTheme::industrial();
    
    // Set background
    let bg_block = Block::default().style(theme.background);
    f.render_widget(bg_block, f.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(" MJVENTOY - INDUSTRIAL MACHINE INTERFACE ")
        .style(theme.text_header)
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(theme.border_active));
    f.render_widget(header, chunks[0]);

    // Footer
    let footer = Paragraph::new(" [Q] QUIT | [TAB] NEXT PANE | [ENTER] SELECT | [VIM-BINDINGS ENABLED] ")
        .style(theme.text_primary)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(theme.border_inactive));
    f.render_widget(footer, chunks[2]);

    match app.state {
        AppState::AuthSelection => render_auth_selection(f, chunks[1], app, &theme),
        AppState::DeviceSelection => render_device_selection(f, chunks[1], app, &theme),
        AppState::Configuration => render_config(f, chunks[1], app, &theme),
        AppState::Installation => render_installation(f, chunks[1], app, &theme),
    }
}

fn render_auth_selection(f: &mut Frame, area: Rect, app: &App, theme: &SteelboreTheme) {
    let items: Vec<ListItem> = app.auth_tools.iter().enumerate().map(|(i, tool)| {
        let style = if i == app.selected_auth_idx { theme.highlight } else { theme.text_primary };
        ListItem::new(tool.as_str()).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().title(" SELECT ROOT TOOL ").borders(Borders::ALL).border_style(theme.border_active))
        .highlight_style(theme.highlight);
    f.render_widget(list, area);
}

fn render_device_selection(f: &mut Frame, area: Rect, app: &App, theme: &SteelboreTheme) {
    let items: Vec<ListItem> = app.devices.iter().enumerate().map(|(i, dev)| {
        let style = if i == app.selected_device_idx { theme.highlight } else { theme.text_primary };
        let content = format!("{} | {} | {}", dev.name, dev.model.as_deref().unwrap_or("Unknown"), dev.size);
        ListItem::new(content).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().title(" SELECT TARGET DEVICE ").borders(Borders::ALL).border_style(theme.border_active))
        .highlight_style(theme.highlight);
    f.render_widget(list, area);
}

fn render_config(f: &mut Frame, area: Rect, app: &App, theme: &SteelboreTheme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3), // New constraint for Dry Run
            Constraint::Min(0),
        ])
        .split(area);

    let secure_boot = Paragraph::new(format!(" SECURE BOOT: {}", if app.secure_boot { "ENABLED" } else { "DISABLED" }))
        .style(if app.selected_config_idx == 0 { theme.highlight } else { theme.text_primary })
        .block(Block::default().borders(Borders::ALL).border_style(theme.border_inactive));
    f.render_widget(secure_boot, chunks[0]);

    let part_style = Paragraph::new(format!(" PARTITION STYLE: {}", if app.use_gpt { "GPT" } else { "MBR" }))
        .style(if app.selected_config_idx == 1 { theme.highlight } else { theme.text_primary })
        .block(Block::default().borders(Borders::ALL).border_style(theme.border_inactive));
    f.render_widget(part_style, chunks[1]);

    let reserve = Paragraph::new(format!(" RESERVE SPACE: {} MiB", app.reserve_space))
        .style(if app.selected_config_idx == 2 { theme.highlight } else { theme.text_primary })
        .block(Block::default().borders(Borders::ALL).border_style(theme.border_inactive));
    f.render_widget(reserve, chunks[2]);

    let dry_run = Paragraph::new(format!(" DRY RUN: {}", if app.dry_run { "[X] (Safe testing)" } else { "[ ] (Destructive)" }))
        .style(if app.selected_config_idx == 3 { theme.highlight } else { theme.text_primary })
        .block(Block::default().borders(Borders::ALL).border_style(theme.border_inactive));
    f.render_widget(dry_run, chunks[3]);

    let install = Paragraph::new(" [ INSTALL VENTOY ] ")
        .style(if app.selected_config_idx == 4 { theme.highlight } else { theme.text_header })
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(theme.border_active));
    f.render_widget(install, chunks[4]);

}


fn render_installation(f: &mut Frame, area: Rect, app: &App, theme: &SteelboreTheme) {

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let gauge = Gauge::default()
        .block(Block::default().title(" PROGRESS ").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Rgb(254, 107, 0)).bg(Color::Rgb(20, 46, 70)))
        .percent((app.progress * 100.0) as u16)
        .label(format!("{:.1}%", app.progress * 100.0));
    f.render_widget(gauge, chunks[0]);

    let log_items: Vec<ListItem> = app.logs.iter().rev().take(10).map(|l| {
        ListItem::new(l.as_str()).style(theme.text_primary)
    }).collect();

    let logs = List::new(log_items)
        .block(Block::default().title(" INSTALLATION LOG ").borders(Borders::ALL).border_style(theme.border_inactive));
    f.render_widget(logs, chunks[1]);

}

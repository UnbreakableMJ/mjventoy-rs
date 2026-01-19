// SPDX-License-Identifier: GPL-3.0-or-later
// Mohamed Hammad
//
// Trademarks:
//   Steelbore, S3cure, S3cure me, S3cure us
//   These names are trademarks of Mohamed Hammad and may not be used
//   to endorse or promote derivative products without prior permission.

use ratatui::style::{Color, Style, Modifier};

pub struct SteelboreTheme {
    pub background: Style,
    pub border_active: Style,
    pub border_inactive: Style,
    pub text_primary: Style,
    pub text_header: Style,
    pub highlight: Style,
}

impl SteelboreTheme {
    pub fn industrial() -> Self {
        Self {
            // Charcoal Navy Canvas
            background: Style::default().bg(Color::Rgb(14, 20, 29)),

            // Vibrant Orange Active Borders (Bold)
            border_active: Style::default()
                .fg(Color::Rgb(254, 107, 0))
                .add_modifier(Modifier::BOLD),

            // Midnight Blue Inactive Borders
            border_inactive: Style::default().fg(Color::Rgb(20, 46, 70)),

            // Silver Grey Body Text
            text_primary: Style::default().fg(Color::Rgb(192, 192, 192)),

            // Vibrant Orange Headers (Bold)
            text_header: Style::default()
                .fg(Color::Rgb(254, 107, 0))
                .add_modifier(Modifier::BOLD),

            // High Contrast Selection (Orange BG, Navy Text)
            highlight: Style::default()
                .bg(Color::Rgb(254, 107, 0))
                .fg(Color::Rgb(14, 20, 29))
                .add_modifier(Modifier::BOLD),
        }
    }
}

use ratatui::style::{Color, Modifier, Style};

/// Theme configuration for consistent styling across all TUI components.
///
/// This provides centralized color and style definitions to ensure all pickers
/// have a consistent appearance matching the tmux switch command style.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    /// Primary color for labels and titles (Cyan)
    pub primary: Color,
    /// Secondary color for selection highlight (Yellow)
    pub secondary: Color,
    /// Success color for confirm actions (Green)
    pub success: Color,
    /// Danger color for cancel actions (Red)
    pub danger: Color,
    /// Info color for hints (Magenta)
    pub info: Color,
    /// Muted color for secondary text (DarkGray)
    pub muted: Color,
    /// Border color (Gray)
    pub border: Color,
    /// Modifier for selected items (Bold)
    pub selected_modifier: Modifier,
}

impl Theme {
    /// Get the default theme matching the tmux switch command style.
    pub fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Yellow,
            success: Color::Green,
            danger: Color::Red,
            info: Color::Magenta,
            muted: Color::DarkGray,
            border: Color::Gray,
            selected_modifier: Modifier::BOLD,
        }
    }

    /// Style for selected items (Yellow + Bold)
    pub fn selected_style(&self) -> Style {
        Style::default()
            .fg(self.secondary)
            .add_modifier(self.selected_modifier)
    }

    /// Style for primary labels (Cyan)
    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.primary)
    }

    /// Style for success/confirm actions (Green)
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Style for danger/cancel actions (Red)
    pub fn danger_style(&self) -> Style {
        Style::default().fg(self.danger)
    }

    /// Style for info/hints (Magenta)
    pub fn info_style(&self) -> Style {
        Style::default().fg(self.info)
    }

    /// Style for muted/secondary text (DarkGray)
    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted)
    }

    /// Style for borders (Gray)
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Style for titles (Cyan + Bold)
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::default()
    }
}

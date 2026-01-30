use iced::widget::container;
use iced::{Color, Theme};
use iced::theme::{Palette, Custom};
use std::sync::Arc;

// Colors
pub const ORANGE: Color = Color::from_rgb(0.984, 0.173, 0.176); // #FB2C2D
pub const GREEN: Color = Color::from_rgb(0.0, 0.78, 0.33); // #00C853
pub const BLUE: Color = Color::from_rgb(0.16, 0.38, 1.0); // #2962FF
pub const PINK: Color = Color::from_rgb(0.77, 0.07, 0.38); // #C51162
pub const YELLOW: Color = Color::from_rgb(1.0, 0.84, 0.0); // #FFD600
pub const CYAN: Color = Color::from_rgb(0.0, 0.7, 0.9); // #00B8D4 (Darker Cyan for visibility)
pub const PURPLE: Color = Color::from_rgb(0.5, 0.0, 1.0);
pub const WHITE: Color = Color::WHITE;
pub const BLACK: Color = Color::BLACK;
pub const DARK_BG: Color = Color::from_rgb(0.07, 0.07, 0.07); // #121212
pub const MUTED_GRAY: Color = Color::from_rgb(0.2, 0.2, 0.2); // #333333
pub const TEXT_DIM: Color = Color::from_rgb(0.5, 0.5, 0.5);
pub const SUCCESS_GREEN: Color = Color::from_rgb(0.0, 1.0, 0.0);
pub const SUCCESS_GREEN_LIGHT: Color = Color::from_rgb(0.0, 0.8, 0.0);
pub const DANGER_RED: Color = Color::from_rgb(1.0, 0.0, 0.0);
pub const DANGER_RED_LIGHT: Color = Color::from_rgb(0.8, 0.0, 0.0);
pub const WARNING_YELLOW: Color = Color::from_rgb(1.0, 0.8, 0.0);
pub const WARNING_YELLOW_LIGHT: Color = Color::from_rgb(0.9, 0.7, 0.0);
pub const TRANSPARENT: Color = Color::TRANSPARENT;

pub fn create_theme(dark_mode: bool, primary: Color) -> Theme {
    let background = if dark_mode { DARK_BG } else { WHITE };
    let adjusted_primary = ensure_readable(primary, background);

    let palette = if dark_mode {
        Palette {
            background,
            text: WHITE,
            primary: adjusted_primary,
            success: SUCCESS_GREEN,
            danger: DANGER_RED,
            warning: WARNING_YELLOW,
        }
    } else {
        Palette {
            background,
            text: MUTED_GRAY,
            primary: adjusted_primary,
            success: SUCCESS_GREEN_LIGHT,
            danger: DANGER_RED_LIGHT,
            warning: WARNING_YELLOW_LIGHT,
        }
    };

    Theme::Custom(Arc::new(Custom::new("Custom".to_string(), palette)))
}

fn ensure_readable(foreground: Color, background: Color) -> Color {
    let fg_lum = luminance(foreground);
    let bg_lum = luminance(background);

    let contrast = if fg_lum > bg_lum {
        (fg_lum + 0.05) / (bg_lum + 0.05)
    } else {
        (bg_lum + 0.05) / (fg_lum + 0.05)
    };

    if contrast >= 4.5 {
        return foreground;
    }

    // Simple adjustment: Lighten or Darken
    // If background is dark (low lum), we need lighter fg.
    // If background is light (high lum), we need darker fg.

    if bg_lum < 0.5 {
        // Dark background -> Lighten
        let mut new_color = foreground;
        for _ in 0..5 {
            new_color.r = (new_color.r + 0.1).min(1.0);
            new_color.g = (new_color.g + 0.1).min(1.0);
            new_color.b = (new_color.b + 0.1).min(1.0);
            if luminance(new_color) > bg_lum { // Re-check contrast
                 let l1 = luminance(new_color);
                 let ratio = (l1 + 0.05) / (bg_lum + 0.05);
                 if ratio >= 4.5 { return new_color; }
            }
        }
        foreground // Return original if fails (or maybe WHITE?)
    } else {
         // Light background -> Darken
        let mut new_color = foreground;
        for _ in 0..5 {
            new_color.r = (new_color.r - 0.1).max(0.0);
            new_color.g = (new_color.g - 0.1).max(0.0);
            new_color.b = (new_color.b - 0.1).max(0.0);
            if luminance(new_color) < bg_lum {
                 let l1 = luminance(new_color);
                 let ratio = (bg_lum + 0.05) / (l1 + 0.05);
                 if ratio >= 4.5 { return new_color; }
            }
        }
        foreground
    }
}

fn luminance(color: Color) -> f32 {
    0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b
}

pub fn container_default(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    container::Style {
        background: Some(palette.background.into()),
        text_color: Some(palette.text),
        ..container::Style::default()
    }
}

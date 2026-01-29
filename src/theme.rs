use iced::widget::container;
use iced::{Color, Theme};
use iced::theme::{Palette, Custom};
use std::sync::Arc;

// Colors
pub const ORANGE: Color = Color::from_rgb(0.984, 0.173, 0.176); // #FB2C2D
pub const CYAN: Color = Color::from_rgb(0.0, 1.0, 1.0);
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
    let palette = if dark_mode {
        Palette {
            background: DARK_BG,
            text: WHITE,
            primary,
            success: SUCCESS_GREEN,
            danger: DANGER_RED,
            warning: WARNING_YELLOW,
        }
    } else {
        Palette {
            background: WHITE,
            text: MUTED_GRAY,
            primary,
            success: SUCCESS_GREEN_LIGHT,
            danger: DANGER_RED_LIGHT,
            warning: WARNING_YELLOW_LIGHT,
        }
    };

    Theme::Custom(Arc::new(Custom::new("Custom".to_string(), palette)))
}

pub fn container_default(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    container::Style {
        background: Some(palette.background.into()),
        text_color: Some(palette.text),
        ..container::Style::default()
    }
}

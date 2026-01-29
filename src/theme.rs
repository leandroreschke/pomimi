use iced::widget::container;
use iced::{Color, Theme};
use iced::theme::{Palette, Custom};
use std::sync::Arc;

// Colors
pub const ORANGE: Color = Color::from_rgb(0.984, 0.173, 0.176); // #FB2C2D
pub const WHITE: Color = Color::WHITE;
pub const DARK_BG: Color = Color::from_rgb(0.07, 0.07, 0.07); // #121212
pub const MUTED_GRAY: Color = Color::from_rgb(0.2, 0.2, 0.2); // #333333
pub const TEXT_DIM: Color = Color::from_rgb(0.5, 0.5, 0.5);
pub const CYAN: Color = Color::from_rgb(0.0, 1.0, 1.0);

pub fn create_theme(dark_mode: bool, primary: Color) -> Theme {
    let palette = if dark_mode {
        Palette {
            background: DARK_BG,
            text: WHITE,
            primary,
            success: Color::from_rgb(0.0, 1.0, 0.0),
            danger: Color::from_rgb(1.0, 0.0, 0.0),
            warning: Color::from_rgb(1.0, 0.8, 0.0),
        }
    } else {
        Palette {
            background: WHITE,
            text: MUTED_GRAY,
            primary,
            success: Color::from_rgb(0.0, 0.8, 0.0),
            danger: Color::from_rgb(0.8, 0.0, 0.0),
            warning: Color::from_rgb(0.9, 0.7, 0.0),
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

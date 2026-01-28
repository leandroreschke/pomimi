use iced::widget::{button, container, progress_bar};
use iced::{Border, Color, Theme, Background};
use iced::theme::{Palette, Custom};
use std::sync::Arc;

// Colors
pub const ORANGE: Color = Color::from_rgb(0.984, 0.173, 0.176); // #FB2C2D
pub const WHITE: Color = Color::WHITE;
pub const DARK_BG: Color = Color::from_rgb(0.07, 0.07, 0.07); // #121212
pub const MUTED_GRAY: Color = Color::from_rgb(0.2, 0.2, 0.2); // #333333
pub const TEXT_DIM: Color = Color::from_rgb(0.5, 0.5, 0.5);
pub const CYAN: Color = Color::from_rgb(0.0, 1.0, 1.0);

pub const PRIMARY: Color = ORANGE;

pub fn create_theme(dark_mode: bool, primary: Color) -> Theme {
    let palette = if dark_mode {
        Palette {
            background: DARK_BG,
            text: WHITE,
            primary,
            success: Color::from_rgb(0.0, 1.0, 0.0),
            danger: Color::from_rgb(1.0, 0.0, 0.0),
        }
    } else {
        Palette {
            background: WHITE,
            text: MUTED_GRAY,
            primary,
            success: Color::from_rgb(0.0, 0.8, 0.0),
            danger: Color::from_rgb(0.8, 0.0, 0.0),
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

pub fn button_primary(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();
    let base = button::Style {
        background: Some(palette.primary.into()),
        text_color: if is_light(&palette.primary) { Color::BLACK } else { Color::WHITE },
        border: Border {
            radius: 0.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(scale_alpha(palette.primary, 0.8).into()),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(scale_alpha(palette.primary, 0.6).into()),
            ..base
        },
        _ => base,
    }
}

pub fn button_secondary(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();
    let base = button::Style {
        background: None,
        text_color: palette.text,
        border: Border {
            color: palette.text,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..button::Style::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette.text.into()),
            text_color: palette.background,
            ..base
        },
        button::Status::Pressed => button::Style {
             background: Some(scale_alpha(palette.text, 0.8).into()),
             ..base
        },
        _ => base,
    }
}

// Minimal/Ghost button
pub fn button_ghost(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();
    let base = button::Style {
        background: None,
        text_color: palette.text,
        border: Border::default(),
        ..button::Style::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            text_color: palette.primary,
            ..base
        },
        _ => base
    }
}

pub fn button_outline(theme: &Theme, status: button::Status) -> button::Style {
    button_secondary(theme, status)
}

pub fn progress_bar_style(theme: &Theme) -> progress_bar::Style {
    let palette = theme.palette();
    progress_bar::Style {
        background: Background::Color(scale_alpha(palette.text, 0.1)),
        bar: Background::Color(palette.primary),
        border: Border {
            radius: 0.0.into(),
            ..Border::default()
        },
    }
}

fn is_light(color: &Color) -> bool {
    color.r * 0.299 + color.g * 0.587 + color.b * 0.114 > 0.5
}

fn scale_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

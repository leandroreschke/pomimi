use crate::theme;
use iced::widget::button;
use iced::{Border, Color, Theme};

pub fn primary(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();
    let base = button::Style {
        background: Some(palette.primary.into()),
        text_color: if is_light(&palette.primary) {
            theme::BLACK
        } else {
            theme::WHITE
        },
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

pub fn secondary(theme: &Theme, status: button::Status) -> button::Style {
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
            background: Some(palette.primary.into()),
            text_color: if is_light(&palette.primary) {
                theme::BLACK
            } else {
                theme::WHITE
            },
            border: Border {
                color: palette.primary,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(scale_alpha(palette.primary, 0.8).into()),
            text_color: if is_light(&palette.primary) {
                theme::BLACK
            } else {
                theme::WHITE
            },
            border: Border {
                color: palette.primary,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..base
        },
        _ => base,
    }
}

pub fn tertiary(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();
    let base = button::Style {
        background: None,
        text_color: palette.text,
        border: Border::default(),
        ..button::Style::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette.primary.into()),
            text_color: if is_light(&palette.primary) {
                theme::BLACK
            } else {
                theme::WHITE
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(scale_alpha(palette.primary, 0.8).into()),
            text_color: if is_light(&palette.primary) {
                theme::BLACK
            } else {
                theme::WHITE
            },
            ..base
        },
        _ => base,
    }
}

fn is_light(color: &Color) -> bool {
    color.r * 0.299 + color.g * 0.587 + color.b * 0.114 > 0.5
}

fn scale_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

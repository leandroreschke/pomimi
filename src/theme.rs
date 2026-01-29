use iced::widget::{button, container, progress_bar};
use iced::{Border, Color, Theme, Background};

pub const BACKGROUND: Color = Color::from_rgb(0.07, 0.07, 0.07);
pub const SURFACE: Color = Color::from_rgb(0.12, 0.12, 0.12);
pub const PRIMARY: Color = Color::from_rgb(1.0, 0.2, 0.6);
pub const ACCENT: Color = Color::from_rgb(0.0, 1.0, 1.0);
pub const TEXT: Color = Color::from_rgb(0.9, 0.9, 0.9);
pub const TEXT_DIM: Color = Color::from_rgb(0.5, 0.5, 0.5);

pub fn container_default(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(BACKGROUND.into()),
        text_color: Some(TEXT),
        ..container::Style::default()
    }
}

pub fn container_bordered(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(SURFACE.into()),
        border: Border {
            color: SURFACE,
            width: 1.0,
            radius: 4.0.into(),
        },
        text_color: Some(TEXT),
        ..container::Style::default()
    }
}

pub fn button_primary(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: PRIMARY,
        border: Border {
            color: PRIMARY,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..button::Style::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(PRIMARY.into()),
            text_color: Color::WHITE,
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Color::from_rgb(0.8, 0.1, 0.5).into()),
            text_color: Color::WHITE,
            ..base
        },
        _ => base,
    }
}

pub fn button_secondary(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: ACCENT,
        border: Border {
            color: ACCENT,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..button::Style::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(ACCENT.into()),
            text_color: Color::BLACK,
            ..base
        },
        button::Status::Pressed => button::Style {
             background: Some(Color::from_rgb(0.0, 0.8, 0.8).into()),
             text_color: Color::BLACK,
             ..base
        },
        _ => base,
    }
}

pub fn progress_bar_style(_theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: Background::Color(SURFACE),
        bar: Background::Color(PRIMARY),
        border: Border {
            color: SURFACE,
            width: 0.0,
            radius: 2.0.into(),
        },
    }
}

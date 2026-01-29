use iced::{Element, Theme, Color};
use iced::widget::{button, container, Space};
use crate::theme;

pub fn checkbox<'a, Message: Clone + 'a>(
    is_checked: bool,
    primary_color: Color,
    on_toggle: Message,
) -> Element<'a, Message> {
    button(
        container(
            if is_checked {
                container(Space::new().width(8).height(8))
                    .style(move |_t: &Theme| container::Style { 
                        background: Some(primary_color.into()), 
                        ..container::Style::default() 
                    })
            } else {
                container(Space::new().width(8).height(8))
            }
        )
        .width(24)
        .height(24)
        .align_x(iced::Alignment::Center)
        .align_y(iced::Alignment::Center)
        .style(move |_t: &Theme| container::Style {
            background: None,
            border: iced::Border {
                color: theme::TEXT_DIM,
                width: 2.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
    )
    .padding(0)
    .style(move |_theme: &Theme, status: button::Status| {
        let base = button::Style {
            background: None,
            text_color: Color::TRANSPARENT,
            border: iced::Border::default(),
            ..button::Style::default()
        };
        match status {
            button::Status::Hovered => button::Style {
                background: Some(Color { a: 0.1, ..primary_color }.into()),
                ..base
            },
            _ => base,
        }
    })
    .on_press(on_toggle)
    .into()
}

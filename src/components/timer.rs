use iced::{Element, Length, Color};
use iced::widget::{column, text, button, container, Space, stack, row};
use crate::theme;
use crate::gui::{Message, State, Phase, ViewMode};

pub fn timer_display<'a>(state: &'a State) -> Element<'a, Message> {
    let mins = state.timer.remaining_secs / 60;
    let secs = state.timer.remaining_secs % 60;
    let time_str = format!("{:02}:{:02}", mins, secs);

    let phase_label = match state.timer.phase {
        Phase::Focus => "FOCUS",
        Phase::ShortBreak | Phase::LongBreak => "REST",
    };

    let timer_display: Element<'a, Message> = if state.view_mode == ViewMode::Full {
        container(
            stack![
                container(
                    text(phase_label)
                        .size(80)
                        .font(iced::Font { family: iced::font::Family::Name("Space Grotesk"), weight: iced::font::Weight::Black, ..iced::Font::DEFAULT })
                        .color(Color { a: 0.05, ..if state.is_dark_mode { theme::WHITE } else { theme::DARK_BG } })
                )
                .align_x(iced::Alignment::Center)
                .align_y(iced::Alignment::Start)
                .padding(iced::Padding { top: -56.0, right: 0.0, bottom: 0.0, left: 0.0 })
                .width(Length::Fill)
                .height(Length::Shrink),
                container(
                    text(time_str)
                        .size(100)
                        .font(iced::Font { family: iced::font::Family::Name("Space Grotesk"), weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT })
                        .line_height(0.9)
                )
                .align_x(iced::Alignment::Center)
                .align_y(iced::Alignment::Center)
                .width(Length::Fill)
                .height(Length::Shrink)
            ]
        )
        .padding(iced::Padding { top: 24.0, right: 0.0, bottom: 72.0, left: 0.0 })
        .into()
    } else {
        text(time_str)
            .size(60)
            .font(iced::Font { family: iced::font::Family::Name("Space Grotesk"), weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT })
            .line_height(0.9)
            .into()
    };

    let mut col = column![timer_display].align_x(iced::Alignment::Center);

    // Show strategy buttons only if NOT running
    if !state.timer.is_running && state.view_mode == ViewMode::Full {
         col = col.push(
             row![
                 button(text("25/5").size(12)).on_press(Message::SetDuration(25*60)).style(crate::components::button::secondary).padding(5),
                 button(text("50/10").size(12)).on_press(Message::SetDuration(50*60)).style(crate::components::button::secondary).padding(5),
             ].spacing(10).padding(10)
         );
    }

    if state.view_mode == ViewMode::Full {
         col = col.push(Space::new().height(20));
         col = col.push(
             button(
                 row![
                     text(if state.timer.waiting_for_user {
                         match state.timer.phase {
                             Phase::Focus => "READY TO FOCUS",
                             _ => "READY TO REST"
                         }
                     } else if state.timer.is_running {
                         "PAUSE"
                     } else {
                         match state.timer.phase {
                             Phase::Focus => "FOCUS",
                             _ => "REST"
                         }
                     }).size(14).font(iced::Font::MONOSPACE).color(theme::BLACK),
                     text("\u{e5c8}").font(iced::Font::with_name("Material Symbols Outlined")).size(14).color(theme::BLACK) // arrow_forward
                 ].spacing(10).align_y(iced::Alignment::Center)
             )
             .width(Length::Fill)
             .padding(15)
             .style(crate::components::button::primary)
             .on_press(Message::ToggleTimer)
         );
    }

    col.into()
}

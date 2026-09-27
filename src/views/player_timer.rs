use crate::state::{Msg, Plaza, TimerMsg};
use crate::theme;
use crate::views::bevel::bevel_button;
use crate::views::{action_close_row, button, format_time, BOLD};
use iced::widget::{column, container, row, text, text_input, Space};
use iced::{Element, Fill, Length};
use std::time::Instant;

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let timer = &state.timer;

    let body: Element<Msg> = match timer.until {
        Some(until) => {
            let remaining = until.saturating_duration_since(Instant::now());
            column![
                text("Sleep Timer").size(11).center().width(Fill),
                Space::new().height(8),
                text(format_time(remaining.as_secs_f64()))
                    .size(14)
                    .font(BOLD)
                    .center()
                    .width(Fill),
            ]
            .width(Fill)
            .into()
        }
        None => {
            let step = |label, delta| {
                container(button(label, Fill).on_press(Msg::Timer(TimerMsg::Add(delta))))
                    .width(Length::FillPortion(2))
            };
            let minutes = text_input("", &timer.minutes_input)
                .on_input(|s| Msg::Timer(TimerMsg::Input(s)))
                .on_submit(Msg::Timer(TimerMsg::Start))
                .size(11)
                .padding([3, 4])
                .align_x(iced::alignment::Horizontal::Center)
                .style(theme::page_input);

            let stepper = row![
                step("-10", -10),
                Space::new().width(4),
                step("-5", -5),
                Space::new().width(4),
                container(minutes).width(Length::FillPortion(4)),
                Space::new().width(4),
                step("+5", 5),
                Space::new().width(4),
                step("+10", 10),
            ]
            .align_y(iced::Alignment::Center);

            column![
                text("Set a timer to automatically stop playback.")
                    .size(11)
                    .center()
                    .width(Fill),
                Space::new().height(12),
                stepper,
            ]
            .width(Fill)
            .into()
        }
    };

    let (label, msg) = if timer.until.is_some() {
        ("Stop Timer", TimerMsg::Stop)
    } else {
        ("Start Timer", TimerMsg::Start)
    };
    let action_btn = bevel_button(text(label).size(11).font(BOLD).center().width(Fill))
        .on_press(Msg::Timer(msg))
        .width(Fill);

    column![
        body,
        Space::new().height(16),
        action_close_row(action_btn, wid),
    ]
    .padding(12)
    .width(Fill)
    .into()
}

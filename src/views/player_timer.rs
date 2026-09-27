use super::bevel::bevel_button;
use super::widgets::{BOLD, action_close_row, button, format_duration};
use crate::message::{Msg, TimerMsg};
use crate::state::Plaza;
use crate::theme;
use iced::widget::{Space, column, container, row, text, text_input};
use iced::window::Id;
use iced::{Element, Fill, Length};
use std::time::Instant;

pub fn view(state: &Plaza, wid: Id) -> Element<'_, Msg> {
    let timer = &state.timer;
    let (body, label, msg) = if let Some(until) = timer.until {
        (countdown(until), "Stop Timer", TimerMsg::Stop)
    } else {
        (setup(&timer.minutes_input), "Start Timer", TimerMsg::Start)
    };
    let action_button = bevel_button(text(label).size(11).font(BOLD).center().width(Fill))
        .on_press(Msg::Timer(msg))
        .width(Fill);

    column![
        body,
        Space::new().height(16),
        action_close_row(action_button, wid),
    ]
    .padding(12)
    .width(Fill)
    .into()
}

fn countdown<'a>(until: Instant) -> Element<'a, Msg> {
    let remaining = until.saturating_duration_since(Instant::now());
    column![
        text("Sleep Timer").size(11).center().width(Fill),
        Space::new().height(8),
        text(format_duration(remaining.as_secs_f64()))
            .size(14)
            .font(BOLD)
            .center()
            .width(Fill),
    ]
    .width(Fill)
    .into()
}

fn setup(minutes_input: &str) -> Element<'_, Msg> {
    let step = |label, delta| {
        container(button(label, Fill).on_press(Msg::Timer(TimerMsg::Add(delta))))
            .width(Length::FillPortion(2))
    };
    let minutes = text_input("", minutes_input)
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

use crate::state::{Msg, Plaza, TimerMsg};
use crate::theme;
use crate::views::bevel_button;
use crate::views::{action_close_row, bold_font, format_time};
use iced::widget::{column, container, row, text, text_input, Space};
use iced::{Element, Fill, Length};
use std::time::Instant;

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let bold = bold_font();
    let active = state.timer.until.is_some();

    let body: Element<Msg> = if let Some(until) = state.timer.until {
        let remaining = until
            .saturating_duration_since(Instant::now())
            .as_secs_f64();
        column![
            text("Sleep Timer").size(11).center().width(Fill),
            Space::new().height(8),
            text(format_time(remaining))
                .size(14)
                .font(bold)
                .center()
                .width(Fill),
        ]
        .width(Fill)
        .into()
    } else {
        let step = |label: &'static str, delta: i32| {
            bevel_button(text(label).size(11).center().width(Fill))
                .on_press(Msg::Timer(TimerMsg::Add(delta)))
                .width(Fill)
        };

        let minutes_input = text_input("", &state.timer.minutes_input)
            .on_input(|s| Msg::Timer(TimerMsg::Input(s)))
            .on_submit(Msg::Timer(TimerMsg::Start))
            .size(11)
            .padding([3, 4])
            .align_x(iced::alignment::Horizontal::Center)
            .style(theme::page_input);

        let stepper = row![
            container(step("-10", -10)).width(Length::FillPortion(2)),
            Space::new().width(4),
            container(step("-5", -5)).width(Length::FillPortion(2)),
            Space::new().width(4),
            container(minutes_input).width(Length::FillPortion(4)),
            Space::new().width(4),
            container(step("+5", 5)).width(Length::FillPortion(2)),
            Space::new().width(4),
            container(step("+10", 10)).width(Length::FillPortion(2)),
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
    };

    let (action_label, action_msg) = if active {
        ("Stop Timer", Msg::Timer(TimerMsg::Stop))
    } else {
        ("Start Timer", Msg::Timer(TimerMsg::Start))
    };
    let action_btn = bevel_button(text(action_label).size(11).font(bold).center().width(Fill))
        .on_press(action_msg)
        .width(Fill);

    column![
        body,
        Space::new().height(16),
        action_close_row(action_btn, wid)
    ]
    .padding(12)
    .width(Fill)
    .into()
}

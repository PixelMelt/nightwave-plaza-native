use crate::state::Msg;
use crate::theme;
use crate::views::{button, d3_sunken, BOLD};
use iced::widget::{column, container, rich_text, span, Space};
use iced::{Element, Fill, Length};

pub fn view(wid: iced::window::Id) -> Element<'static, Msg> {
    let para1: iced::widget::text::Rich<'_, (), Msg> = rich_text![
        span("Nightwave Plaza").font(BOLD).size(12),
        span(" website and apps are created and maintained by ").size(12),
        span("Alexander Morozov")
            .color(theme::LINK_COLOR)
            .underline(true)
            .size(12),
        span(".").size(12),
    ];

    let para2: iced::widget::text::Rich<'_, (), Msg> = rich_text![
        span("All music and backgrounds").font(BOLD).size(12),
        span(" belong to their respective authors. Musical content is provided by artists and labels. If you have any copyright concerns, please let us know.").size(12),
    ];

    let memo = d3_sunken(
        container(column![para1, Space::new().height(8), para2].padding(6))
            .style(theme::sunken_inner)
            .width(Fill)
            .padding(4),
    );

    let close = button("Close", Length::Shrink)
        .on_press(Msg::CloseWin(wid))
        .padding([4, 24]);
    let bottom = container(close).width(Fill).center_x(Fill).padding([8, 0]);

    column![memo, bottom].padding(8).into()
}

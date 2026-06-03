use crate::state::Msg;
use crate::theme;
use crate::views::widgets::{bold_font, close_btn_padded, d3_sunken, LINK_COLOR};
use iced::widget::{column, container, rich_text, span, Space};
use iced::{Element, Fill};

pub fn view(wid: iced::window::Id) -> Element<'static, Msg> {
    let bold = bold_font();

    let para1: iced::widget::text::Rich<'_, (), Msg> = rich_text![
        span("Nightwave Plaza").font(bold).size(12),
        span(" website and apps are created and maintained by ").size(12),
        span("Alexander Morozov")
            .color(LINK_COLOR)
            .underline(true)
            .size(12),
        span(".").size(12),
    ];

    let para2: iced::widget::text::Rich<'_, (), Msg> = rich_text![
        span("All music and backgrounds").font(bold).size(12),
        span(" belong to their respective authors. Musical content is provided by artists and labels. If you have any copyright concerns, please let us know.").size(12),
    ];

    let memo_content = column![para1, Space::new().height(8), para2]
        .spacing(0)
        .padding(6);

    let memo = d3_sunken(
        container(memo_content)
            .style(theme::sunken_inner)
            .width(Fill)
            .padding(4),
    );

    let bottom = container(close_btn_padded(wid))
        .width(Fill)
        .center_x(Fill)
        .padding([8, 0]);

    column![memo, bottom].padding([8, 8]).into()
}

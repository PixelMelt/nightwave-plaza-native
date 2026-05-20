use crate::state::Msg;
use crate::theme;
use crate::views::widgets::{d3_raised, d3_sunken};
use iced::widget::{button, column, container, rich_text, span, text, Space};
use iced::{Element, Fill};

const LINK_COLOR: iced::Color = iced::Color::from_rgb(0.024, 0.271, 0.678); // #0645AD

pub fn view(wid: iced::window::Id) -> Element<'static, Msg> {
    let bold = iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::DEFAULT
    };

    // win-memo: sunken text area
    // Paragraph 1: "{app} website and apps are created and maintained by {author}."
    // app = Nightwave Plaza (bold), author = Alexander Morozov (link-colored)
    let para1 = rich_text![
        span("Nightwave Plaza").font(bold).size(12),
        span(" website and apps are created and maintained by ").size(12),
        span("Alexander Morozov")
            .color(LINK_COLOR)
            .underline(true)
            .size(12),
        span(".").size(12),
    ];

    // Paragraph 2: "{content} belong to their respective authors..."
    // content = "All music and backgrounds" (bold)
    let para2 = rich_text![
        span("All music and backgrounds").font(bold).size(12),
        span(" belong to their respective authors. Musical content is provided by artists and labels. If you have any copyright concerns, please let us know.").size(12),
    ];

    let memo_content = column![para1, Space::with_height(8), para2]
        .spacing(0)
        .padding(6);

    let memo = d3_sunken(
        container(memo_content)
            .style(theme::sunken_inner)
            .width(Fill)
            .padding(4),
    );

    // Close button centered
    let close_btn = d3_raised(
        button(text("Close").size(11).center())
            .on_press(Msg::CloseWin(wid))
            .style(theme::raised)
            .padding([4, 24]),
    );

    let bottom = container(close_btn)
        .width(Fill)
        .center_x(Fill)
        .padding([8, 0]);

    column![memo, bottom].padding([8, 8]).into()
}

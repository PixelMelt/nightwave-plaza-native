use crate::state::Msg;
use crate::theme;
use crate::views::widgets::{bold_font, d3_sunken, raised_btn, status_bar, LINK_COLOR};
use iced::widget::{column, container, horizontal_space, image, row, text, Space};
use iced::{Element, Fill, Padding};

const PC_IMG: &[u8] = include_bytes!("../assets/img/pc.png");

pub fn view(wid: iced::window::Id) -> Element<'static, Msg> {
    let bold = bold_font();
    let bold_italic = iced::Font {
        weight: iced::font::Weight::Bold,
        style: iced::font::Style::Italic,
        ..iced::Font::DEFAULT
    };
    let italic = iced::Font {
        style: iced::font::Style::Italic,
        ..iced::Font::DEFAULT
    };

    let lh = iced::widget::text::LineHeight::Relative(1.5);

    let title_col = column![
        text("Nightwave Plaza")
            .size(14)
            .font(bold_italic)
            .center()
            .width(Fill),
        Space::with_height(4),
        text("Welcome to the 24/7 online vaporwave and future funk radio station.")
            .size(12)
            .font(italic)
            .line_height(lh)
            .center()
            .width(Fill),
    ]
    .width(Fill)
    .align_x(iced::Alignment::Center);

    let pc_image = image(image::Handle::from_bytes(PC_IMG)).width(70);

    let top_row = row![title_col, pc_image]
        .spacing(8)
        .padding([4, 6])
        .align_y(iced::Alignment::Center);

    let panel_body = column![
        text("Contact Information")
            .size(12)
            .font(bold)
            .line_height(lh),
        Space::with_height(4),
        text("Please send any inquiries you may have to mail@plaza.one.")
            .size(12)
            .line_height(lh),
        Space::with_height(4),
        text("Join our community Discord server!")
            .size(12)
            .line_height(lh),
        Space::with_height(8),
        text("Submissions").size(12).font(bold).line_height(lh),
        Space::with_height(4),
        text("Want to submit music for broadcast? Please use this form.")
            .size(12)
            .line_height(lh),
        Space::with_height(8),
        text("Mobile applications (iOS / Android)")
            .size(12)
            .font(bold)
            .line_height(lh),
        text("Show more").size(12).line_height(lh).color(LINK_COLOR),
        Space::with_height(8),
        text("Useful links").size(12).font(bold).line_height(lh),
        Space::with_height(4),
        text("Playlists").size(12).line_height(lh),
        row![
            text("M3U (Winamp)")
                .size(12)
                .line_height(lh)
                .color(LINK_COLOR),
            Space::with_width(12),
            text("PLS (Foobar2000)")
                .size(12)
                .line_height(lh)
                .color(LINK_COLOR),
        ],
        Space::with_height(8),
        text("Streams").size(12).line_height(lh),
        text("http://radio.plaza.one/mp3 (mp3 / 128kbps)")
            .size(12)
            .line_height(lh)
            .color(LINK_COLOR),
        text("http://radio.plaza.one/ogg (opus / 96kbps)")
            .size(12)
            .line_height(lh)
            .color(LINK_COLOR),
        text("http://radio.plaza.one/hls (hls / aac)")
            .size(12)
            .line_height(lh)
            .color(LINK_COLOR),
    ]
    .spacing(0);

    let panel = d3_sunken(
        container(panel_body)
            .style(theme::panel)
            .width(Fill)
            .padding(8),
    );

    let pad = Padding::from([4, 24]);
    let credits_btn = raised_btn(
        "Credits",
        Msg::OpenWin(crate::state::WinType::Credits),
        iced::Length::Shrink,
        pad,
    );
    let news_btn = raised_btn(
        "News",
        Msg::OpenWin(crate::state::WinType::News),
        iced::Length::Shrink,
        pad,
    );
    let close_btn = raised_btn("Close", Msg::CloseWin(wid), iced::Length::Shrink, pad);

    let bottom = row![
        credits_btn,
        Space::with_width(8),
        news_btn,
        horizontal_space(),
        close_btn,
    ];

    let version = env!("CARGO_PKG_VERSION");
    let status = status_bar(vec![text(format!("Version: {}", version))
        .size(10)
        .width(Fill)
        .into()]);

    column![
        top_row,
        Space::with_height(8),
        panel,
        Space::with_height(16),
        bottom,
        Space::with_height(2),
        status,
    ]
    .spacing(0)
    .padding(8)
    .into()
}

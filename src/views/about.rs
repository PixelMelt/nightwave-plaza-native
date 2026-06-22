use crate::state::Msg;
use crate::theme;
use crate::views::{
    bold_font, d3_sunken, flat_button_style, raised_btn, static_image, status_bar, LINK_COLOR,
};
use iced::widget::{button, column, container, image, row, text, Space};
use iced::{Element, Fill, Padding};

const PC_IMG: &[u8] = include_bytes!("../assets/img/pc.png");

fn about_link(label: &'static str, url: &'static str) -> iced::widget::Button<'static, Msg> {
    button(
        text(label)
            .size(12)
            .line_height(iced::widget::text::LineHeight::Relative(1.5))
            .color(LINK_COLOR),
    )
    .on_press(Msg::OpenUrl(url.into()))
    .style(|_, _| flat_button_style(LINK_COLOR))
    .padding(0)
}

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
        Space::new().height(4),
        text("Welcome to the 24/7 online vaporwave and future funk radio station.")
            .size(12)
            .font(italic)
            .line_height(lh)
            .center()
            .width(Fill),
    ]
    .width(Fill)
    .align_x(iced::Alignment::Center);

    let pc_image = image(static_image(PC_IMG)).width(70);

    let top_row = row![title_col, pc_image]
        .spacing(8)
        .padding([4, 6])
        .align_y(iced::Alignment::Center);

    let panel_body = column![
        text("Contact Information")
            .size(12)
            .font(bold)
            .line_height(lh),
        Space::new().height(4),
        text("Please send any inquiries you may have to mail@plaza.one.")
            .size(12)
            .line_height(lh),
        Space::new().height(4),
        text("Join our community Discord server!")
            .size(12)
            .line_height(lh),
        Space::new().height(8),
        text("Submissions").size(12).font(bold).line_height(lh),
        Space::new().height(4),
        text("Want to submit music for broadcast? Please use this form.")
            .size(12)
            .line_height(lh),
        Space::new().height(8),
        text("Mobile applications (iOS / Android)")
            .size(12)
            .font(bold)
            .line_height(lh),
        about_link("Show more", "https://plaza.one"),
        Space::new().height(8),
        text("Useful links").size(12).font(bold).line_height(lh),
        Space::new().height(4),
        text("Playlists").size(12).line_height(lh),
        row![
            about_link("M3U (Winamp)", "https://radio.plaza.one/mp3.m3u"),
            Space::new().width(12),
            about_link("PLS (Foobar2000)", "https://plaza.one/plaza.pls"),
        ],
        Space::new().height(8),
        text("Streams").size(12).line_height(lh),
        about_link(
            "http://radio.plaza.one/mp3 (mp3 / 128kbps)",
            "http://radio.plaza.one/mp3",
        ),
        about_link(
            "http://radio.plaza.one/ogg (opus / 96kbps)",
            "http://radio.plaza.one/ogg",
        ),
        about_link(
            "http://radio.plaza.one/hls (hls / aac)",
            "http://radio.plaza.one/hls",
        ),
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
        Space::new().width(8),
        news_btn,
        Space::new().width(iced::Fill),
        close_btn,
    ];

    let version = env!("CARGO_PKG_VERSION");
    let status = status_bar(vec![text(format!("Version: {}", version))
        .size(10)
        .width(Fill)
        .into()]);

    column![
        top_row,
        Space::new().height(8),
        panel,
        Space::new().height(16),
        bottom,
        Space::new().height(2),
        status,
    ]
    .spacing(0)
    .padding(8)
    .into()
}

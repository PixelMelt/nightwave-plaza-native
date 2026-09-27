use super::widgets::{BOLD, CellWidth, Png, button, link_button, status_bar, sunken_panel};
use crate::message::Msg;
use crate::window::WindowKind;
use iced::widget::text::LineHeight;
use iced::widget::{Space, column, row, text};
use iced::window::Id;
use iced::{Element, Fill, Font, Length};

static PC: Png = Png::new(include_bytes!("../assets/img/pc.png"));

const BOLD_ITALIC: Font = Font {
    weight: iced::font::Weight::Bold,
    style: iced::font::Style::Italic,
    ..Font::DEFAULT
};
const ITALIC: Font = Font {
    style: iced::font::Style::Italic,
    ..Font::DEFAULT
};
const LH: LineHeight = LineHeight::Relative(1.5);

fn link(label: &'static str, url: &'static str) -> Element<'static, Msg> {
    link_button(label, 12, Some(Msg::OpenUrl(url.into())))
}

fn heading(label: &'static str) -> iced::widget::Text<'static> {
    text(label).size(12).font(BOLD).line_height(LH)
}

fn line(label: &'static str) -> iced::widget::Text<'static> {
    text(label).size(12).line_height(LH)
}

pub fn view(wid: Id) -> Element<'static, Msg> {
    let title_col = column![
        text("Nightwave Plaza")
            .size(14)
            .font(BOLD_ITALIC)
            .center()
            .width(Fill),
        Space::new().height(4),
        text("Welcome to the 24/7 online vaporwave and future funk radio station.")
            .size(12)
            .font(ITALIC)
            .line_height(LH)
            .center()
            .width(Fill),
    ]
    .width(Fill)
    .align_x(iced::Alignment::Center);

    let top_row = row![title_col, PC.image().width(70)]
        .spacing(8)
        .padding([4, 6])
        .align_y(iced::Alignment::Center);

    let panel = sunken_panel(column![
        heading("Contact Information"),
        Space::new().height(4),
        line("Please send any inquiries you may have to mail@plaza.one."),
        Space::new().height(4),
        line("Join our community Discord server!"),
        Space::new().height(8),
        heading("Submissions"),
        Space::new().height(4),
        line("Want to submit music for broadcast? Please use this form."),
        Space::new().height(8),
        heading("Mobile applications (iOS / Android)"),
        link("Show more", "https://plaza.one"),
        Space::new().height(8),
        heading("Useful links"),
        Space::new().height(4),
        line("Playlists"),
        row![
            link("M3U (Winamp)", "https://radio.plaza.one/mp3.m3u"),
            Space::new().width(12),
            link("PLS (Foobar2000)", "https://plaza.one/plaza.pls"),
        ],
        Space::new().height(8),
        line("Streams"),
        link(
            "http://radio.plaza.one/mp3 (mp3 / 128kbps)",
            "http://radio.plaza.one/mp3",
        ),
        link(
            "http://radio.plaza.one/ogg (opus / 96kbps)",
            "http://radio.plaza.one/ogg",
        ),
        link(
            "http://radio.plaza.one/hls (hls / aac)",
            "http://radio.plaza.one/hls",
        ),
    ]);

    let wide = |label, msg| button(label, Length::Shrink).on_press(msg).padding([4, 24]);
    let bottom = row![
        wide("Credits", Msg::OpenWindow(WindowKind::Credits)),
        Space::new().width(8),
        wide("News", Msg::OpenWindow(WindowKind::News)),
        Space::new().width(Fill),
        wide("Close", Msg::CloseWindow(wid)),
    ];

    let status = status_bar(vec![(
        text(format!("Version: {}", env!("CARGO_PKG_VERSION")))
            .size(10)
            .into(),
        CellWidth::Portion(1),
    )]);

    column![
        top_row,
        Space::new().height(8),
        panel,
        Space::new().height(16),
        bottom,
        Space::new().height(2),
        status,
    ]
    .padding(8)
    .into()
}

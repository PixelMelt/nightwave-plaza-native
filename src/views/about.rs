use crate::state::Msg;
use crate::theme;
use crate::views::widgets::{d3_raised, d3_sunken, status_bar};
use iced::widget::{button, column, container, horizontal_space, image, row, text, Space};
use iced::{Element, Fill};

const PC_IMG: &[u8] = include_bytes!("../../assets/icons/pc.png");

pub fn view(wid: iced::window::Id) -> Element<'static, Msg> {
    let bold = iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::DEFAULT
    };
    let bold_italic = iced::Font {
        weight: iced::font::Weight::Bold,
        style: iced::font::Style::Italic,
        ..iced::Font::DEFAULT
    };
    let italic = iced::Font {
        style: iced::font::Style::Italic,
        ..iced::Font::DEFAULT
    };

    // All <p> in #window-about: font-size: 12px; line-height: 150%
    let lh = iced::widget::text::LineHeight::Relative(1.5);

    // ── Top row (mb-2 = 8px below) ──────────────────────────────
    // Left col: text-center aligned
    let title_col = column![
        // p.lead: 14px, bold italic, mb-1 = 4px
        text("Nightwave Plaza")
            .size(14)
            .font(bold_italic)
            .center()
            .width(Fill),
        Space::with_height(4), // mb-1
        // <p><i>welcome text</i></p> — italic 12px
        text("Welcome to the 24/7 online vaporwave and future funk radio station.")
            .size(12)
            .font(italic)
            .line_height(lh)
            .center()
            .width(Fill),
    ]
    .width(Fill)
    .align_x(iced::Alignment::Center);

    // Right col: pc.png 70px, vertically centered within the row
    let pc_image = image(image::Handle::from_bytes(PC_IMG)).width(70);

    let top_row = row![title_col, pc_image]
        .spacing(8)
        .padding([4, 6])
        .align_y(iced::Alignment::Center);

    // ── Panel (win-panel, mb-3 = 16px below) ────────────────────
    // No explicit padding on win-panel; <p> elements sit directly inside
    let panel_body = column![
        // <p><strong>Contact Information</strong></p>
        text("Contact Information")
            .size(12)
            .font(bold)
            .line_height(lh),
        Space::with_height(4), // default <p> margin-bottom 0.25rem
        // <p>Please send any inquiries you may have to mail@plaza.one.</p>
        text("Please send any inquiries you may have to mail@plaza.one.")
            .size(12)
            .line_height(lh),
        Space::with_height(4), // default <p> margin
        // <p class="mb-2">Join our community Discord server!</p>
        text("Join our community Discord server!")
            .size(12)
            .line_height(lh),
        Space::with_height(8), // mb-2
        // <p><strong>Submissions</strong></p>
        text("Submissions").size(12).font(bold).line_height(lh),
        Space::with_height(4),
        // <p class="mb-2">Want to submit music for broadcast? Please use this form.</p>
        text("Want to submit music for broadcast? Please use this form.")
            .size(12)
            .line_height(lh),
        Space::with_height(8), // mb-2
        // <p class="mb-2"><strong>Mobile applications (iOS / Android)</strong><br><a>Show more</a></p>
        text("Mobile applications (iOS / Android)")
            .size(12)
            .font(bold)
            .line_height(lh),
        text("Show more")
            .size(12)
            .line_height(lh)
            .color(iced::Color::from_rgb(0.024, 0.271, 0.678)), // link color #0645AD
        Space::with_height(8), // mb-2
        // <p><strong>Useful links</strong></p>
        text("Useful links").size(12).font(bold).line_height(lh),
        Space::with_height(4),
        // <p class="mb-2">Playlists<br>M3U (Winamp)  PLS (Foobar2000)</p>
        text("Playlists").size(12).line_height(lh),
        row![
            text("M3U (Winamp)")
                .size(12)
                .line_height(lh)
                .color(iced::Color::from_rgb(0.024, 0.271, 0.678)),
            Space::with_width(12), // ms-3
            text("PLS (Foobar2000)")
                .size(12)
                .line_height(lh)
                .color(iced::Color::from_rgb(0.024, 0.271, 0.678)),
        ],
        Space::with_height(8), // mb-2
        // <p>Streams<br>...</p>
        text("Streams").size(12).line_height(lh),
        text("http://radio.plaza.one/mp3 (mp3 / 128kbps)")
            .size(12)
            .line_height(lh)
            .color(iced::Color::from_rgb(0.024, 0.271, 0.678)),
        text("http://radio.plaza.one/ogg (opus / 96kbps)")
            .size(12)
            .line_height(lh)
            .color(iced::Color::from_rgb(0.024, 0.271, 0.678)),
        text("http://radio.plaza.one/hls (hls / aac)")
            .size(12)
            .line_height(lh)
            .color(iced::Color::from_rgb(0.024, 0.271, 0.678)),
    ]
    .spacing(0);

    let panel = d3_sunken(
        container(panel_body)
            .style(theme::panel)
            .width(Fill)
            .padding(8),
    );

    let credits_btn = d3_raised(
        button(text("Credits").size(11).center())
            .on_press(Msg::OpenWin(crate::state::WinType::Credits))
            .style(theme::raised)
            .padding([4, 24]),
    );
    let news_btn = d3_raised(
        button(text("News").size(11).center())
            .on_press(Msg::OpenWin(crate::state::WinType::News))
            .style(theme::raised)
            .padding([4, 24]),
    );
    let close_btn = d3_raised(
        button(text("Close").size(11).center())
            .on_press(Msg::CloseWin(wid))
            .style(theme::raised)
            .padding([4, 24]),
    );

    let bottom = row![
        credits_btn,
        Space::with_width(8),
        news_btn,
        horizontal_space(),
        close_btn,
    ];

    // ── Status bar: "Version: {version}" ────────────────────────
    let version = env!("CARGO_PKG_VERSION");
    let status = status_bar(vec![text(format!("Version: {}", version))
        .size(10)
        .width(Fill)
        .into()]);

    // Outer layout: p-2 = 8px padding
    column![
        top_row,
        Space::with_height(8), // mb-2 on top row
        panel,
        Space::with_height(16), // mb-3 on panel
        bottom,
        Space::with_height(2),
        status,
    ]
    .spacing(0)
    .padding(8) // p-2
    .into()
}

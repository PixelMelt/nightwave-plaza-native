use crate::state::{Msg, Plaza};
use crate::theme;
use crate::views::widgets::{d3_raised, d3_sunken, format_date, pagination};
use iced::widget::{
    button, column, container, horizontal_space, rich_text, row, scrollable, span, text, Column,
    Space,
};
use iced::{Element, Fill, Font};

const LOADING_IMG: &[u8] = include_bytes!("../../assets/icons/loading.png");

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<Msg> {
    let bold = Font {
        weight: iced::font::Weight::Bold,
        ..Font::DEFAULT
    };

    // Content area: win-memo style (sunken, monospace font, scrollable)
    let content_area: Element<Msg> = if state.news_loading {
        d3_sunken(
            container(
                iced::widget::image(iced::widget::image::Handle::from_bytes(LOADING_IMG))
                    .width(36)
                    .height(36),
            )
            .style(theme::sunken_inner)
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill),
        )
        .width(Fill)
        .height(Fill)
        .into()
    } else if state.news.is_empty() {
        d3_sunken(
            container(text("No news.").size(11))
                .style(theme::sunken_inner)
                .width(Fill)
                .height(Fill)
                .center_x(Fill)
                .center_y(Fill),
        )
        .width(Fill)
        .height(Fill)
        .into()
    } else {
        let mut articles = Column::new().spacing(0).width(Fill);

        for (idx, article) in state.news.iter().enumerate() {
            if article.text.is_empty() {
                continue;
            }

            // Parse HTML into blocks and render
            let blocks = parse_html_blocks(&article.text);
            let mut article_col = Column::new().spacing(0).padding([6, 6]).width(Fill);

            for block in &blocks {
                match block {
                    HtmlBlock::Heading(txt) => {
                        // h2: font-size: 14px; margin-bottom: 8px
                        article_col = article_col
                            .push(text(txt.clone()).size(14).font(bold))
                            .push(Space::with_height(8));
                    }
                    HtmlBlock::Paragraph(segments) => {
                        // p: font-size: 11px; margin-bottom: 4px
                        if segments.len() == 1 && !segments[0].1 {
                            article_col = article_col.push(text(segments[0].0.clone()).size(11));
                        } else {
                            let spans: Vec<_> = segments
                                .iter()
                                .map(|(txt, is_bold)| {
                                    let s = span(txt.clone()).size(11);
                                    if *is_bold {
                                        s.font(bold)
                                    } else {
                                        s
                                    }
                                })
                                .collect();
                            article_col = article_col.push(rich_text(spans));
                        }
                        article_col = article_col.push(Space::with_height(4));
                    }
                    HtmlBlock::ListItem(txt) => {
                        // ul li: margin-left: 16px; margin-bottom: 4px
                        let item_row = row![
                            Space::with_width(16),
                            text(format!("\u{2022} {}", txt)).size(11)
                        ];
                        article_col = article_col.push(item_row).push(Space::with_height(4));
                    }
                }
            }

            // Author + date row below article
            article_col = article_col.push(Space::with_height(2)).push(row![
                text(article.author.clone())
                    .size(10)
                    .color(iced::Color::from_rgb(0.4, 0.4, 0.4)),
                horizontal_space(),
                text(format_date(article.created_at))
                    .size(10)
                    .color(iced::Color::from_rgb(0.4, 0.4, 0.4)),
            ]);

            articles = articles.push(article_col);

            // Divider between articles
            if idx < state.news.len() - 1 {
                articles = articles.push(container(Space::new(Fill, 1)).style(
                    move |_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(
                            0.78, 0.78, 0.78,
                        ))),
                        ..Default::default()
                    },
                ));
            }
        }

        d3_sunken(
            container(scrollable(articles).height(Fill).style(theme::scrollbar))
                .style(theme::sunken_inner)
                .width(Fill)
                .height(Fill),
        )
        .width(Fill)
        .height(Fill)
        .into()
    };

    // Pagination + Close
    let pages_row = if state.news_pages > 1 {
        let prev_msg = if state.news_page > 1 && !state.news_loading {
            Some(Msg::NewsPage(state.news_page - 1))
        } else {
            None
        };
        let next_msg = if state.news_page < state.news_pages && !state.news_loading {
            Some(Msg::NewsPage(state.news_page + 1))
        } else {
            None
        };
        pagination(
            state.news_page,
            state.news_pages,
            &state.news_page.to_string(),
            state.news_loading,
            |_| Msg::Refresh,
            Msg::Refresh,
            prev_msg,
            next_msg,
        )
    } else {
        Space::with_width(0).into()
    };

    let close_btn = d3_raised(
        button(text("Close").size(11).center().width(Fill))
            .on_press(Msg::CloseWin(wid))
            .style(theme::raised)
            .width(80),
    );

    let bottom = row![pages_row, horizontal_space(), close_btn]
        .align_y(iced::Alignment::Center)
        .padding([4, 0]);

    column![content_area, Space::with_height(4), bottom]
        .padding(8)
        .height(Fill)
        .into()
}

// ── Simple HTML block parser ────────────────────────────────────

enum HtmlBlock {
    /// <h2>text</h2>
    Heading(String),
    /// <p>segments...</p> where segments are (text, is_bold) pairs
    Paragraph(Vec<(String, bool)>),
    /// <li>text</li>
    ListItem(String),
}

fn parse_html_blocks(html: &str) -> Vec<HtmlBlock> {
    let mut blocks = Vec::new();
    let mut pos = 0;
    let bytes = html.as_bytes();

    while pos < html.len() {
        // Skip whitespace
        while pos < html.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= html.len() {
            break;
        }

        if html[pos..].starts_with("<h2") {
            // Find closing </h2>
            if let Some(start) = html[pos..].find('>') {
                let content_start = pos + start + 1;
                if let Some(end) = html[content_start..].find("</h2>") {
                    let inner = &html[content_start..content_start + end];
                    blocks.push(HtmlBlock::Heading(strip_tags(inner)));
                    pos = content_start + end + 5;
                    continue;
                }
            }
        } else if html[pos..].starts_with("<li") {
            if let Some(start) = html[pos..].find('>') {
                let content_start = pos + start + 1;
                if let Some(end) = html[content_start..].find("</li>") {
                    let inner = &html[content_start..content_start + end];
                    blocks.push(HtmlBlock::ListItem(strip_tags(inner)));
                    pos = content_start + end + 5;
                    continue;
                }
            }
        } else if html[pos..].starts_with("<p") {
            if let Some(start) = html[pos..].find('>') {
                let content_start = pos + start + 1;
                if let Some(end) = html[content_start..].find("</p>") {
                    let inner = &html[content_start..content_start + end];
                    blocks.push(HtmlBlock::Paragraph(parse_inline(inner)));
                    pos = content_start + end + 4;
                    continue;
                }
            }
        } else if html[pos..].starts_with("<ul")
            || html[pos..].starts_with("</ul")
            || html[pos..].starts_with("<br")
            || html[pos..].starts_with("\r")
        {
            // Skip block-level wrappers
            if let Some(end) = html[pos..].find('>') {
                pos += end + 1;
                continue;
            }
        }

        // If we didn't match any tag, treat remaining text as a paragraph
        let remaining = html[pos..].trim();
        if !remaining.is_empty() && !remaining.starts_with('<') {
            // Plain text until next tag
            let end = html[pos..].find('<').unwrap_or(html.len() - pos);
            let txt = html[pos..pos + end].trim();
            if !txt.is_empty() {
                blocks.push(HtmlBlock::Paragraph(vec![(txt.to_string(), false)]));
            }
            pos += end;
        } else {
            pos += 1; // skip unrecognized char
        }
    }

    blocks
}

/// Parse inline HTML, extracting <strong>/<b> segments
fn parse_inline(html: &str) -> Vec<(String, bool)> {
    let mut segments = Vec::new();
    let mut pos = 0;

    while pos < html.len() {
        if html[pos..].starts_with("<strong")
            || html[pos..].starts_with("<b>")
            || html[pos..].starts_with("<b ")
        {
            // Find end of opening tag
            if let Some(tag_end) = html[pos..].find('>') {
                let content_start = pos + tag_end + 1;
                let close_tag = if html[pos..].starts_with("<strong") {
                    "</strong>"
                } else {
                    "</b>"
                };
                if let Some(end) = html[content_start..].find(close_tag) {
                    let inner = strip_tags(&html[content_start..content_start + end]);
                    if !inner.is_empty() {
                        segments.push((inner, true));
                    }
                    pos = content_start + end + close_tag.len();
                    continue;
                }
            }
        } else if html[pos..].starts_with('<') {
            // Skip other tags (e.g. <string>, <a>, <br>)
            if let Some(end) = html[pos..].find('>') {
                // Check for <br> - insert line break
                if html[pos..pos + end].starts_with("<br") {
                    segments.push((" ".to_string(), false));
                }
                pos += end + 1;
                continue;
            }
        }

        // Regular text
        let end = html[pos..].find('<').unwrap_or(html.len() - pos);
        let txt = &html[pos..pos + end];
        if !txt.is_empty() {
            segments.push((txt.to_string(), false));
        }
        pos += end;
    }

    if segments.is_empty() {
        segments.push((strip_tags(html), false));
    }

    segments
}

/// Strip all HTML tags from a string
fn strip_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result.trim().to_string()
}

use crate::state::{Msg, Plaza};
use crate::views::widgets::{
    bold_font, close_btn, empty_panel, format_date, loading_panel, pagination, scroll_panel, MUTED,
};
use tl::{Node, Parser};

use iced::widget::{
    column, container, horizontal_space, rich_text, row, span, text, Column, Space,
};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let bold = bold_font();

    let content_area: Element<Msg> = if state.news.loading {
        loading_panel()
    } else if state.news.list.is_empty() {
        empty_panel("No news.")
    } else {
        let mut articles = Column::new().spacing(0).width(Fill);

        for (idx, article) in state.news.list.iter().enumerate() {
            if article.blocks.is_empty() {
                continue;
            }

            let mut article_col = Column::new().spacing(0).padding([6, 6]).width(Fill);

            for block in &article.blocks {
                match block {
                    HtmlBlock::Heading(txt) => {
                        article_col = article_col
                            .push(text(txt.clone()).size(14).font(bold))
                            .push(Space::with_height(8));
                    }
                    HtmlBlock::Paragraph(segments) => {
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
                        let item_row = row![
                            Space::with_width(16),
                            text(format!("\u{2022} {txt}")).size(11)
                        ];
                        article_col = article_col.push(item_row).push(Space::with_height(4));
                    }
                }
            }

            article_col = article_col.push(Space::with_height(2)).push(row![
                text(article.author.clone()).size(10).color(MUTED),
                horizontal_space(),
                text(format_date(article.created_at)).size(10).color(MUTED),
            ]);

            articles = articles.push(article_col);

            if idx < state.news.list.len() - 1 {
                articles = articles.push(container(Space::new(Fill, 1)).style(
                    |_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(
                            0.78, 0.78, 0.78,
                        ))),
                        ..Default::default()
                    },
                ));
            }
        }

        scroll_panel(articles)
    };

    let pages_row = if state.news.pages > 1 {
        let prev_msg = (state.news.page > 1 && !state.news.loading)
            .then_some(Msg::News(crate::state::NewsMsg::Page(state.news.page - 1)));
        let next_msg = (state.news.page < state.news.pages && !state.news.loading)
            .then_some(Msg::News(crate::state::NewsMsg::Page(state.news.page + 1)));
        pagination(
            state.news.page,
            state.news.pages,
            &state.news.page.to_string(),
            state.news.loading,
            |_| Msg::Refresh,
            Msg::Refresh,
            prev_msg,
            next_msg,
        )
    } else {
        Space::with_width(0).into()
    };

    let bottom = row![pages_row, horizontal_space(), close_btn(wid)]
        .align_y(iced::Alignment::Center)
        .padding([4, 0]);

    column![content_area, Space::with_height(4), bottom]
        .padding(8)
        .height(Fill)
        .into()
}

#[derive(Debug, Clone)]
pub struct ParsedNewsArticle {
    pub author: String,
    pub created_at: u64,
    pub blocks: Vec<HtmlBlock>,
}

impl From<crate::api::NewsArticle> for ParsedNewsArticle {
    fn from(article: crate::api::NewsArticle) -> Self {
        Self {
            author: article.author,
            created_at: article.created_at,
            blocks: parse_html_blocks(&article.text),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlBlock {
    Heading(String),
    Paragraph(Vec<(String, bool)>),
    ListItem(String),
}

fn decode_entities(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&apos;", "'")
}

fn collect_inline(
    node: &Node,
    parser: &Parser,
    is_bold: bool,
    segments: &mut Vec<(String, bool)>,
) {
    match node {
        Node::Raw(bytes) => {
            let text = decode_entities(&bytes.as_utf8_str());
            if !text.is_empty() {
                segments.push((text, is_bold));
            }
        }
        Node::Tag(tag) => {
            let tag_name = tag.name().as_utf8_str();
            let current_bold = is_bold || matches!(tag_name.as_ref(), "strong" | "b" | "string");
            let tag_children = tag.children();
            let children = tag_children.top().as_slice();
            if children.is_empty() {
                if tag_name == "br" {
                    segments.push((" ".to_string(), false));
                }
            } else {
                for &handle in children {
                    if let Some(child_node) = handle.get(parser) {
                        collect_inline(child_node, parser, current_bold, segments);
                    }
                }
            }
        }
        Node::Comment(_) => {}
    }
}

fn collect_blocks(node: &Node, parser: &Parser, blocks: &mut Vec<HtmlBlock>) {
    match node {
        Node::Tag(tag) => match tag.name().as_utf8_str().as_ref() {
            "h2" | "li" => {
                let text = decode_entities(&tag.inner_text(parser));
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    blocks.push(if tag.name().as_utf8_str() == "h2" {
                        HtmlBlock::Heading(trimmed.to_string())
                    } else {
                        HtmlBlock::ListItem(trimmed.to_string())
                    });
                }
            }
            "p" => {
                let mut segments = Vec::new();
                for &handle in tag.children().top().as_slice() {
                    if let Some(child_node) = handle.get(parser) {
                        collect_inline(child_node, parser, false, &mut segments);
                    }
                }
                if !segments.is_empty() {
                    blocks.push(HtmlBlock::Paragraph(segments));
                }
            }
            _ => {
                for &handle in tag.children().top().as_slice() {
                    if let Some(child_node) = handle.get(parser) {
                        collect_blocks(child_node, parser, blocks);
                    }
                }
            }
        },
        Node::Raw(bytes) => {
            let text = bytes.as_utf8_str().trim().to_string();
            if !text.is_empty() {
                blocks.push(HtmlBlock::Paragraph(vec![(decode_entities(&text), false)]));
            }
        }
        Node::Comment(_) => {}
    }
}

fn parse_html_blocks(html: &str) -> Vec<HtmlBlock> {
    let mut blocks = Vec::new();
    if let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) {
        let parser = dom.parser();
        for handle in dom.children() {
            if let Some(node) = handle.get(parser) {
                collect_blocks(node, parser, &mut blocks);
            }
        }
    }
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_16() {
        let html = "<p><strong>iOS App update</strong></p>\r\n<p>The iOS app finally got a new beta update. If you want to test it, please join TestFlight using this link:</p>\r\n<p><a href=\"https://plaza.one/ios_beta\">https://plaza.one/ios_beta</a></p>";
        let blocks = parse_html_blocks(html);
        assert_eq!(
            blocks,
            vec![
                HtmlBlock::Paragraph(vec![("iOS App update".to_string(), true)]),
                HtmlBlock::Paragraph(vec![(
                    "The iOS app finally got a new beta update. If you want to test it, please join TestFlight using this link:".to_string(),
                    false
                )]),
                HtmlBlock::Paragraph(vec![("https://plaza.one/ios_beta".to_string(), false)]),
            ]
        );
    }

    #[test]
    fn test_article_14() {
        let html = "<p><strong>My Profile update</strong></p>\r\n<p>Good news — you can now change your username in <string>My Profile</string>. You can also delete your account at any time.</p>";
        let blocks = parse_html_blocks(html);
        assert_eq!(
            blocks,
            vec![
                HtmlBlock::Paragraph(vec![("My Profile update".to_string(), true)]),
                HtmlBlock::Paragraph(vec![
                    ("Good news — you can now change your username in ".to_string(), false),
                    ("My Profile".to_string(), true),
                    (". You can also delete your account at any time.".to_string(), false),
                ]),
            ]
        );
    }

    #[test]
    fn test_article_13() {
        let html = "<p><strong>Login Issue</strong></p>\r\n<p>Login issue has been fixed. You can now try logging into your account again. If you still can’t sign in, please contact support at <a href=\"mailto:mail@plaza.one\">mail@plaza.one</a>.</p>\r\n<p>Thank you for your patience!</p>";
        let blocks = parse_html_blocks(html);
        assert_eq!(
            blocks,
            vec![
                HtmlBlock::Paragraph(vec![("Login Issue".to_string(), true)]),
                HtmlBlock::Paragraph(vec![
                    ("Login issue has been fixed. You can now try logging into your account again. If you still can’t sign in, please contact support at ".to_string(), false),
                    ("mail@plaza.one".to_string(), false),
                    (".".to_string(), false)
                ]),
                HtmlBlock::Paragraph(vec![("Thank you for your patience!".to_string(), false)]),
            ]
        );
    }

    #[test]
    fn test_article_9() {
        let html = "<p><strong>Submissions</strong></p>\r\n<p>Submissions are open again!</p>\r\n<p>Please use the following link to submit your music for broadcast:</p>\r\n<p><a href=\"https://plaza.one/submissions\" target=\"_blank\">https://plaza.one/submissions</a></p>";
        let blocks = parse_html_blocks(html);
        assert_eq!(
            blocks,
            vec![
                HtmlBlock::Paragraph(vec![("Submissions".to_string(), true)]),
                HtmlBlock::Paragraph(vec![("Submissions are open again!".to_string(), false)]),
                HtmlBlock::Paragraph(vec![("Please use the following link to submit your music for broadcast:".to_string(), false)]),
                HtmlBlock::Paragraph(vec![("https://plaza.one/submissions".to_string(), false)]),
            ]
        );
    }

    #[test]
    fn test_article_5() {
        let html = "<p>Hello listeners!</p>\n<p>The website has been updated. New features:</p>\n<ul>\n<li>Added the news window.</li>\n<li>Added themes support and custom background colors.</li>\n<li>UI updates and more accurate windows styles.</li>\n</ul>\n<p>The &quot;Dislike&quot; button was removed as it no longer makes any sense.</p>\n<p>We hope you will like the new update.</p>";
        let blocks = parse_html_blocks(html);
        assert_eq!(
            blocks,
            vec![
                HtmlBlock::Paragraph(vec![("Hello listeners!".to_string(), false)]),
                HtmlBlock::Paragraph(vec![("The website has been updated. New features:".to_string(), false)]),
                HtmlBlock::ListItem("Added the news window.".to_string()),
                HtmlBlock::ListItem("Added themes support and custom background colors.".to_string()),
                HtmlBlock::ListItem("UI updates and more accurate windows styles.".to_string()),
                HtmlBlock::Paragraph(vec![
                    ("The \"Dislike\" button was removed as it no longer makes any sense.".to_string(), false)
                ]),
                HtmlBlock::Paragraph(vec![("We hope you will like the new update.".to_string(), false)]),
            ]
        );
    }
}

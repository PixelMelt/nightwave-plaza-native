use crate::api::NewsArticle;
use tl::{HTMLTag, Node, Parser};

#[derive(Debug, Clone)]
pub struct Article {
    pub author: String,
    pub created_at: u64,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading(String),
    Paragraph(Vec<Span>),
    ListItem(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub bold: bool,
}

impl From<NewsArticle> for Article {
    fn from(article: NewsArticle) -> Self {
        Self {
            author: article.author,
            created_at: article.created_at,
            blocks: parse_blocks(&article.text),
        }
    }
}

fn decode_entities(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn children<'a>(tag: &HTMLTag<'a>, parser: &'a Parser<'a>) -> Vec<&'a Node<'a>> {
    tag.children()
        .top()
        .iter()
        .filter_map(|handle| handle.get(parser))
        .collect()
}

fn collect_spans(node: &Node, parser: &Parser, bold: bool, spans: &mut Vec<Span>) {
    match node {
        Node::Raw(bytes) => {
            let text = decode_entities(&bytes.as_utf8_str());
            if !text.is_empty() {
                spans.push(Span { text, bold });
            }
        }
        Node::Tag(tag) => {
            let name = tag.name().as_utf8_str();
            let bold = bold || matches!(name.as_ref(), "strong" | "b" | "string");
            if name == "br" {
                spans.push(Span {
                    text: " ".into(),
                    bold: false,
                });
            }
            for child in children(tag, parser) {
                collect_spans(child, parser, bold, spans);
            }
        }
        Node::Comment(_) => {}
    }
}

fn collect_blocks(node: &Node, parser: &Parser, blocks: &mut Vec<Block>) {
    match node {
        Node::Tag(tag) => match tag.name().as_utf8_str().as_ref() {
            name @ ("h2" | "li") => {
                let text = decode_entities(&tag.inner_text(parser));
                let text = text.trim();
                if text.is_empty() {
                    return;
                }
                blocks.push(if name == "h2" {
                    Block::Heading(text.into())
                } else {
                    Block::ListItem(text.into())
                });
            }
            "p" => {
                let mut spans = Vec::new();
                for child in children(tag, parser) {
                    collect_spans(child, parser, false, &mut spans);
                }
                if !spans.is_empty() {
                    blocks.push(Block::Paragraph(spans));
                }
            }
            _ => {
                for child in children(tag, parser) {
                    collect_blocks(child, parser, blocks);
                }
            }
        },
        Node::Raw(bytes) => {
            let text = bytes.as_utf8_str();
            let text = text.trim();
            if !text.is_empty() {
                blocks.push(Block::Paragraph(vec![Span {
                    text: decode_entities(text),
                    bold: false,
                }]));
            }
        }
        Node::Comment(_) => {}
    }
}

fn parse_blocks(html: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    if let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) {
        let parser = dom.parser();
        for node in dom.children().iter().filter_map(|h| h.get(parser)) {
            collect_blocks(node, parser, &mut blocks);
        }
    }
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(text: &str) -> Span {
        Span {
            text: text.into(),
            bold: false,
        }
    }

    fn bold(text: &str) -> Span {
        Span {
            text: text.into(),
            bold: true,
        }
    }

    #[test]
    fn link_paragraph_keeps_link_text() {
        let html = "<p><strong>iOS App update</strong></p>\r\n<p>The iOS app finally got a new beta update. If you want to test it, please join TestFlight using this link:</p>\r\n<p><a href=\"https://plaza.one/ios_beta\">https://plaza.one/ios_beta</a></p>";
        assert_eq!(
            parse_blocks(html),
            vec![
                Block::Paragraph(vec![bold("iOS App update")]),
                Block::Paragraph(vec![plain(
                    "The iOS app finally got a new beta update. If you want to test it, please join TestFlight using this link:"
                )]),
                Block::Paragraph(vec![plain("https://plaza.one/ios_beta")]),
            ]
        );
    }

    #[test]
    fn misspelled_string_tag_is_bold() {
        let html = "<p><strong>My Profile update</strong></p>\r\n<p>Good news — you can now change your username in <string>My Profile</string>. You can also delete your account at any time.</p>";
        assert_eq!(
            parse_blocks(html),
            vec![
                Block::Paragraph(vec![bold("My Profile update")]),
                Block::Paragraph(vec![
                    plain("Good news — you can now change your username in "),
                    bold("My Profile"),
                    plain(". You can also delete your account at any time."),
                ]),
            ]
        );
    }

    #[test]
    fn inline_link_splits_paragraph() {
        let html = "<p><strong>Login Issue</strong></p>\r\n<p>Login issue has been fixed. You can now try logging into your account again. If you still can’t sign in, please contact support at <a href=\"mailto:mail@plaza.one\">mail@plaza.one</a>.</p>\r\n<p>Thank you for your patience!</p>";
        assert_eq!(
            parse_blocks(html),
            vec![
                Block::Paragraph(vec![bold("Login Issue")]),
                Block::Paragraph(vec![
                    plain(
                        "Login issue has been fixed. You can now try logging into your account again. If you still can’t sign in, please contact support at "
                    ),
                    plain("mail@plaza.one"),
                    plain("."),
                ]),
                Block::Paragraph(vec![plain("Thank you for your patience!")]),
            ]
        );
    }

    #[test]
    fn link_attributes_are_ignored() {
        let html = "<p><strong>Submissions</strong></p>\r\n<p>Submissions are open again!</p>\r\n<p>Please use the following link to submit your music for broadcast:</p>\r\n<p><a href=\"https://plaza.one/submissions\" target=\"_blank\">https://plaza.one/submissions</a></p>";
        assert_eq!(
            parse_blocks(html),
            vec![
                Block::Paragraph(vec![bold("Submissions")]),
                Block::Paragraph(vec![plain("Submissions are open again!")]),
                Block::Paragraph(vec![plain(
                    "Please use the following link to submit your music for broadcast:"
                )]),
                Block::Paragraph(vec![plain("https://plaza.one/submissions")]),
            ]
        );
    }

    #[test]
    fn lists_and_entities() {
        let html = "<p>Hello listeners!</p>\n<p>The website has been updated. New features:</p>\n<ul>\n<li>Added the news window.</li>\n<li>Added themes support and custom background colors.</li>\n<li>UI updates and more accurate windows styles.</li>\n</ul>\n<p>The &quot;Dislike&quot; button was removed as it no longer makes any sense.</p>\n<p>We hope you will like the new update.</p>";
        assert_eq!(
            parse_blocks(html),
            vec![
                Block::Paragraph(vec![plain("Hello listeners!")]),
                Block::Paragraph(vec![plain("The website has been updated. New features:")]),
                Block::ListItem("Added the news window.".into()),
                Block::ListItem("Added themes support and custom background colors.".into()),
                Block::ListItem("UI updates and more accurate windows styles.".into()),
                Block::Paragraph(vec![plain(
                    "The \"Dislike\" button was removed as it no longer makes any sense."
                )]),
                Block::Paragraph(vec![plain("We hope you will like the new update.")]),
            ]
        );
    }

    #[test]
    fn escaped_ampersand_is_decoded_once() {
        assert_eq!(decode_entities("&amp;lt;b&amp;gt;"), "&lt;b&gt;");
    }
}

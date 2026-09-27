use crate::api::NewsArticle;
use tl::{Node, Parser};

#[derive(Debug, Clone)]
pub struct Article {
    pub author: String,
    pub created_at: u64,
    pub blocks: Vec<HtmlBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlBlock {
    Heading(String),
    Paragraph(Vec<(String, bool)>),
    ListItem(String),
}

impl From<NewsArticle> for Article {
    fn from(article: NewsArticle) -> Self {
        Self {
            author: article.author,
            created_at: article.created_at,
            blocks: parse_html_blocks(&article.text),
        }
    }
}

fn decode_entities(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&apos;", "'")
}

fn children<'a>(tag: &tl::HTMLTag<'a>, parser: &'a Parser<'a>) -> Vec<&'a Node<'a>> {
    tag.children()
        .top()
        .iter()
        .filter_map(|handle| handle.get(parser))
        .collect()
}

fn collect_inline(node: &Node, parser: &Parser, is_bold: bool, segments: &mut Vec<(String, bool)>) {
    match node {
        Node::Raw(bytes) => {
            let text = decode_entities(&bytes.as_utf8_str());
            if !text.is_empty() {
                segments.push((text, is_bold));
            }
        }
        Node::Tag(tag) => {
            let name = tag.name().as_utf8_str();
            let bold = is_bold || matches!(name.as_ref(), "strong" | "b" | "string");
            if name == "br" {
                segments.push((" ".to_string(), false));
            }
            for child in children(tag, parser) {
                collect_inline(child, parser, bold, segments);
            }
        }
        Node::Comment(_) => {}
    }
}

fn collect_blocks(node: &Node, parser: &Parser, blocks: &mut Vec<HtmlBlock>) {
    match node {
        Node::Tag(tag) => match tag.name().as_utf8_str().as_ref() {
            name @ ("h2" | "li") => {
                let text = decode_entities(&tag.inner_text(parser));
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    blocks.push(if name == "h2" {
                        HtmlBlock::Heading(trimmed.to_string())
                    } else {
                        HtmlBlock::ListItem(trimmed.to_string())
                    });
                }
            }
            "p" => {
                let mut segments = Vec::new();
                for child in children(tag, parser) {
                    collect_inline(child, parser, false, &mut segments);
                }
                if !segments.is_empty() {
                    blocks.push(HtmlBlock::Paragraph(segments));
                }
            }
            _ => {
                for child in children(tag, parser) {
                    collect_blocks(child, parser, blocks);
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
        for node in dom.children().iter().filter_map(|h| h.get(parser)) {
            collect_blocks(node, parser, &mut blocks);
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
                    (
                        "Good news — you can now change your username in ".to_string(),
                        false
                    ),
                    ("My Profile".to_string(), true),
                    (
                        ". You can also delete your account at any time.".to_string(),
                        false
                    ),
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
                HtmlBlock::Paragraph(vec![(
                    "Please use the following link to submit your music for broadcast:".to_string(),
                    false
                )]),
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
                HtmlBlock::Paragraph(vec![(
                    "The website has been updated. New features:".to_string(),
                    false
                )]),
                HtmlBlock::ListItem("Added the news window.".to_string()),
                HtmlBlock::ListItem(
                    "Added themes support and custom background colors.".to_string()
                ),
                HtmlBlock::ListItem("UI updates and more accurate windows styles.".to_string()),
                HtmlBlock::Paragraph(vec![(
                    "The \"Dislike\" button was removed as it no longer makes any sense."
                        .to_string(),
                    false
                )]),
                HtmlBlock::Paragraph(vec![(
                    "We hope you will like the new update.".to_string(),
                    false
                )]),
            ]
        );
    }
}

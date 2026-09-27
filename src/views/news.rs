use crate::news::{Article, HtmlBlock};
use crate::state::{Msg, NewsMsg, Plaza};
use crate::theme::MUTED;
use crate::views::{
    close_btn, empty_panel, format_date, loading_panel, paginate, scroll_panel, separator, BOLD,
};
use iced::widget::{column, rich_text, row, span, text, Column, Space};
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: iced::window::Id) -> Element<'_, Msg> {
    let content = if state.news.pager.loading {
        loading_panel()
    } else if state.news.list.is_empty() {
        empty_panel("No news.")
    } else {
        let mut articles = Column::new().width(Fill);
        for (i, article) in state.news.list.iter().enumerate() {
            if i > 0 {
                articles = articles.push(separator());
            }
            articles = articles.push(article_view(article));
        }
        scroll_panel(articles)
    };

    let bottom = row![
        paginate(&state.news.pager, |m| Msg::News(NewsMsg::Page(m))),
        Space::new().width(Fill),
        close_btn(wid),
    ]
    .align_y(iced::Alignment::Center)
    .padding([4, 0]);

    column![content, Space::new().height(4), bottom]
        .padding(8)
        .height(Fill)
        .into()
}

fn article_view(article: &Article) -> Element<'_, Msg> {
    let mut col = Column::new().padding(6).width(Fill);
    for block in &article.blocks {
        col = match block {
            HtmlBlock::Heading(txt) => col
                .push(text(txt).size(14).font(BOLD))
                .push(Space::new().height(8)),
            HtmlBlock::Paragraph(segments) => {
                let spans = segments.iter().map(|(txt, bold)| {
                    let s = span::<(), _>(txt.as_str()).size(11);
                    if *bold {
                        s.font(BOLD)
                    } else {
                        s
                    }
                });
                col.push(rich_text(spans.collect::<Vec<_>>()))
                    .push(Space::new().height(4))
            }
            HtmlBlock::ListItem(txt) => col
                .push(row![
                    Space::new().width(16),
                    text(format!("\u{2022} {txt}")).size(11)
                ])
                .push(Space::new().height(4)),
        };
    }
    col.push(Space::new().height(2))
        .push(row![
            text(&article.author).size(10).color(MUTED),
            Space::new().width(Fill),
            text(format_date(article.created_at)).size(10).color(MUTED),
        ])
        .into()
}

use super::widgets::{
    BOLD, close_button, empty_panel, format_date, loading_panel, paginate, scroll_panel, separator,
};
use crate::article::{Article, Block};
use crate::message::{Msg, NewsMsg};
use crate::state::Plaza;
use crate::theme::MUTED;
use iced::widget::{Column, Space, column, rich_text, row, span, text};
use iced::window::Id;
use iced::{Element, Fill};

pub fn view(state: &Plaza, wid: Id) -> Element<'_, Msg> {
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
        close_button(wid),
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
            Block::Heading(heading) => col
                .push(text(heading).size(14).font(BOLD))
                .push(Space::new().height(8)),
            Block::Paragraph(spans) => {
                let spans: Vec<_> = spans
                    .iter()
                    .map(|s| {
                        let styled = span::<(), _>(s.text.as_str()).size(11);
                        if s.bold { styled.font(BOLD) } else { styled }
                    })
                    .collect();
                col.push(rich_text(spans)).push(Space::new().height(4))
            }
            Block::ListItem(item) => col
                .push(row![
                    Space::new().width(16),
                    text(format!("\u{2022} {item}")).size(11)
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

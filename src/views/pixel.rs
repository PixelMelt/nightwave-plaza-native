use crate::theme;
use crate::views::bevel::quad;
use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget};
use iced::{Color, Element, Length, Rectangle, Size};

enum Glyph {
    Close,
    Minimize,
    DashedLine(Color),
}

pub struct Pixel {
    glyph: Glyph,
    width: Length,
    height: Length,
}

pub fn close_glyph() -> Pixel {
    Pixel {
        glyph: Glyph::Close,
        width: Length::Fixed(9.0),
        height: Length::Fixed(9.0),
    }
}

pub fn minimize_glyph() -> Pixel {
    Pixel {
        glyph: Glyph::Minimize,
        width: Length::Fixed(9.0),
        height: Length::Fixed(9.0),
    }
}

pub fn dashed_line(color: Color) -> Pixel {
    Pixel {
        glyph: Glyph::DashedLine(color),
        width: Length::Fill,
        height: Length::Fixed(1.0),
    }
}

fn dot(renderer: &mut iced::Renderer, x: f32, y: f32, width: f32, height: f32, color: Color) {
    quad(
        renderer,
        Rectangle {
            x,
            y,
            width,
            height,
        },
        color,
    );
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for Pixel {
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let b = layout.bounds();
        match self.glyph {
            Glyph::Close => {
                for i in 0..7 {
                    let o = i as f32;
                    dot(
                        renderer,
                        b.x + 1.0 + o,
                        b.y + 1.0 + o,
                        1.0,
                        1.0,
                        theme::BLACK,
                    );
                    dot(
                        renderer,
                        b.x + 7.0 - o,
                        b.y + 1.0 + o,
                        1.0,
                        1.0,
                        theme::BLACK,
                    );
                }
            }
            Glyph::Minimize => dot(renderer, b.x + 1.0, b.y + 7.0, 6.0, 2.0, theme::BLACK),
            Glyph::DashedLine(color) => {
                let mut x = b.x;
                while x < b.x + b.width {
                    dot(renderer, x, b.y, 2.0f32.min(b.x + b.width - x), 1.0, color);
                    x += 5.0;
                }
            }
        }
    }
}

impl<'a, Message> From<Pixel> for Element<'a, Message> {
    fn from(p: Pixel) -> Self {
        Element::new(p)
    }
}

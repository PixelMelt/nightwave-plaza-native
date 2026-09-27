use super::paint::fill_rect;
use crate::theme;
use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget};
use iced::{Color, Element, Length, Rectangle, Renderer, Size, Theme};

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

impl<Message> Widget<Message, Theme, Renderer> for Pixel {
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let b = layout.bounds();
        let rect = |x, y, width, height| Rectangle {
            x,
            y,
            width,
            height,
        };
        match self.glyph {
            Glyph::Close => {
                for i in 0..7u8 {
                    let offset = f32::from(i);
                    let dot_y = b.y + 1.0 + offset;
                    fill_rect(
                        renderer,
                        rect(b.x + 1.0 + offset, dot_y, 1.0, 1.0),
                        theme::BLACK,
                    );
                    fill_rect(
                        renderer,
                        rect(b.x + 7.0 - offset, dot_y, 1.0, 1.0),
                        theme::BLACK,
                    );
                }
            }
            Glyph::Minimize => {
                fill_rect(renderer, rect(b.x + 1.0, b.y + 7.0, 6.0, 2.0), theme::BLACK);
            }
            Glyph::DashedLine(color) => {
                let right = b.x + b.width;
                let mut x = b.x;
                while x < right {
                    fill_rect(renderer, rect(x, b.y, (right - x).min(2.0), 1.0), color);
                    x += 5.0;
                }
            }
        }
    }
}

impl<Message> From<Pixel> for Element<'_, Message> {
    fn from(pixel: Pixel) -> Self {
        Element::new(pixel)
    }
}

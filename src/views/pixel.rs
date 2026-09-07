use crate::theme;
use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget};
use iced::{Background, Border, Color, Element, Length, Rectangle, Shadow, Size};

#[derive(Clone, Copy)]
pub enum Glyph {
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

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Pixel
where
    Renderer: iced::advanced::Renderer,
{
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
        let quad = |renderer: &mut Renderer, rect: Rectangle, color: Color| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: rect,
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: false,
                },
                Background::Color(color),
            );
        };

        match self.glyph {
            Glyph::Close => {
                // plaza draws the close glyph as 1px SVG lines
                // (1,1)-(8,8) and (8,1)-(1,8) inside a 9x9 box
                for i in 0..7 {
                    let o = i as f32;
                    quad(
                        renderer,
                        Rectangle {
                            x: b.x + 1.0 + o,
                            y: b.y + 1.0 + o,
                            width: 1.0,
                            height: 1.0,
                        },
                        theme::BLACK,
                    );
                    quad(
                        renderer,
                        Rectangle {
                            x: b.x + 7.0 - o,
                            y: b.y + 1.0 + o,
                            width: 1.0,
                            height: 1.0,
                        },
                        theme::BLACK,
                    );
                }
            }
            Glyph::Minimize => {
                quad(
                    renderer,
                    Rectangle {
                        x: b.x + 1.0,
                        y: b.y + 7.0,
                        width: 6.0,
                        height: 2.0,
                    },
                    theme::BLACK,
                );
            }
            Glyph::DashedLine(color) => {
                let mut x = b.x;
                while x < b.x + b.width {
                    quad(
                        renderer,
                        Rectangle {
                            x,
                            y: b.y,
                            width: 2.0f32.min(b.x + b.width - x),
                            height: 1.0,
                        },
                        color,
                    );
                    x += 5.0;
                }
            }
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Pixel> for Element<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn from(p: Pixel) -> Self {
        Element::new(p)
    }
}

use crate::piece::{Piece, PieceColor, PieceType};
use crate::Message;
use iced::Background;
use iced::Color;
use iced::Length;
use iced::Point;
use iced::Rectangle;
use iced::Size;
use iced_graphics::Backend;
use iced_graphics::Primitive;
use iced_graphics::Renderer;
use iced_native::layout;
use iced_native::Element;
use iced_native::Widget;
use iced_native::Renderer as _;
use std::collections::HashMap;

mod state;
mod castling;
mod promotion_menu;

pub use state::BoardState;
pub use state::GameResult;
use promotion_menu::PromotionMenu;

#[derive(Debug, Clone)]
pub struct Handles(HashMap<(PieceType, PieceColor), iced::svg::Handle>);

impl Handles {
    pub fn new() -> Self {
        use PieceType::*;
        let mut inner = HashMap::new();
        for piece in [Pawn, King, Queen, Rook, Bishop, Knight] {
            for color in [PieceColor::White, PieceColor::Black] {
                let prefix = match color {
                    PieceColor::White => "w",
                    PieceColor::Black => "b",
                };
                let postfix = match piece {
                    Pawn => "P",
                    King => "K",
                    Queen => "Q",
                    Rook => "R",
                    Bishop => "B",
                    Knight => "N",
                };
                let path =
                    format!("/Users/ilyabogdanov/projects/chess/resources/{prefix}{postfix}.svg");
                let handle = iced::svg::Handle::from_path(path);
                inner.insert((piece, color), handle);
            }
        }
        Self(inner)
    }

    pub fn get(&self, key: &(PieceType, PieceColor)) -> Option<iced::svg::Handle> {
        self.0.get(key).cloned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Square {
    Empty,
    Piece(Piece),
}

const LIGHT_SQUARE_COLOR: Color = Color {
    r: 245.0 / 255.0,
    g: 245.0 / 255.0,
    b: 245.0 / 255.0,
    a: 1.0,
};
const DARK_SQUARE_COLOR: Color = Color {
    r: 176.0 / 255.0,
    g: 224.0 / 255.0,
    b: 230.0 / 255.0,
    a: 1.0,
};

pub struct Board<'a> {
    state: &'a mut BoardState,
    show_promotion_menu: bool,
    promotion_menu: PromotionMenu,
}

impl<'a> Board<'a> {
    pub fn new(state: &'a mut BoardState) -> Self {
        Self { state, show_promotion_menu: true, promotion_menu: PromotionMenu::new() }
    }
}

impl<'a> Widget<Message, iced_wgpu::Renderer> for Board<'a>
{
    fn width(&self) -> Length {
        Length::Fill
    }
    fn height(&self) -> Length {
        Length::Fill
    }
    fn layout(
        &self,
        renderer: &iced_wgpu::Renderer,
        limits: &iced_native::layout::Limits,
    ) -> iced_native::layout::Node {
        let max_size = limits.max().width.min(limits.max().height);

        layout::Node::new(Size::new(max_size, max_size))
    }
    fn hash_layout(&self, state: &mut iced_native::Hasher) {}
    fn on_event(
        &mut self,
        event: iced_native::Event,
        layout: iced_native::Layout<'_>,
        cursor_position: Point,
        _renderer: &iced_wgpu::Renderer,
        _clipboard: &mut dyn iced_native::Clipboard,
        shell: &mut iced_native::Shell<'_, Message>,
    ) -> iced_native::event::Status {
        match event {
            iced_native::Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Left,
            )) => {
                if layout.bounds().contains(cursor_position) {
                    let size = layout.bounds().size();
                    let bounds = layout.bounds();
                    let x = cursor_position.x - bounds.x;
                    let y = cursor_position.y - bounds.y;
                    let x = x / size.width;
                    let y = y / size.height;
                    let normalized = Point::new(x, y);
                    let pos = self.state.cursor_position_to_pos(normalized);
                    shell.publish(Message::ClickOnSquare(pos));
                    return iced_native::event::Status::Captured;
                }
                iced_native::event::Status::Ignored
            }
            _ => iced_native::event::Status::Ignored,
        }
    }

    fn draw(
        &self,
        renderer: &mut iced_wgpu::Renderer,
        style: &iced_native::renderer::Style,
        layout: iced_native::Layout<'_>,
        cursor_position: iced::Point,
        viewport: &iced::Rectangle,
    ) {
        let b = layout.bounds();
        let square_width = b.width / 8.0;
        let square_height = b.height / 8.0;
        for col in 0..8 {
            for row in 0..8 {
                let color = (col + row) % 2 == 0;
                let color = if color {
                    LIGHT_SQUARE_COLOR
                } else {
                    DARK_SQUARE_COLOR
                };
                let top_left = Point::new(
                    b.x + square_width * col as f32,
                    b.y + square_height * row as f32,
                );
                let size = Size::new(square_width, square_height);
                let bounds = Rectangle::new(top_left, size);
                let primitive = Primitive::Quad {
                    bounds,
                    background: Background::Color(color),
                    border_width: 0.0,
                    border_color: color,
                    border_radius: 0.0,
                };
                renderer.draw_primitive(primitive);
            }
        }
        if let Some((_, pos)) = self.state.selected_piece {
            let pos = Point::new(
                (pos.file.as_u8() - 1) as f32 * square_width,
                (8 - pos.rank.get()) as f32 * square_height,
            );
            let primitive = Primitive::Quad {
                bounds: Rectangle::new(pos, Size::new(square_width, square_height)),
                background: Background::Color(Color::TRANSPARENT),
                border_width: 3.0,
                border_color: Color::new(1.0, 1.0, 0.0, 1.0),
                border_radius: 15.0,
            };
            renderer.draw_primitive(primitive);
        }
        for (i, (square, is_highlighted)) in self.state.squares.iter().enumerate() {
            let y = i / 8;
            let x = i % 8;
            let pos = (x, y);
            let pos = Point::new(pos.0 as f32 * square_width, pos.1 as f32 * square_height);
            if *is_highlighted {
                let primitive = Primitive::Quad {
                    bounds: Rectangle::new(pos, Size::new(square_width, square_height)),
                    background: Background::Color(Color::TRANSPARENT),
                    border_width: 3.0,
                    border_color: Color::new(1.0, 0.0, 0.0, 1.0),
                    border_radius: 10.0,
                };
                renderer.draw_primitive(primitive);
            }
            if let Square::Piece(piece) = square {
                let handle = self.state.handles.get(&(piece.kind, piece.color)).unwrap();
                let primitive = Primitive::Svg {
                    handle,
                    bounds: Rectangle::new(pos, Size::new(square_width, square_height)),
                };
                renderer.draw_primitive(primitive);
            }
        }

        if self.show_promotion_menu {
            renderer.with_layer(*viewport, |renderer| {
                PromotionMenu::new().view().draw(renderer, style, layout, cursor_position, viewport);
            });
        }
    }
}

impl<'a> Into<Element<'a, Message, iced_wgpu::Renderer>> for Board<'a>
{
    fn into(self) -> Element<'a, Message, iced_wgpu::Renderer> {
        Element::new(self)
    }
}

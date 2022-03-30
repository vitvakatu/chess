use crate::piece::{Piece, PieceColor, PieceType};
use crate::pos::Pos;
use std::collections::HashMap;
use yew::prelude::*;

mod castling;
mod state;

pub use state::BoardState;
pub use state::GameResult;
pub use state::IsHighlighted;

use crate::moves::internal::Move;
use crate::moves::CastlingSide;
use crate::moves::PromotedTo;
use crate::pos::{File, Rank};
use crate::Msg;

#[derive(Debug, PartialEq)]
enum HighlightColor {
    Red,
    Yellow,
}

#[derive(Properties, PartialEq)]
struct HighlightProps {
    x: i32,
    y: i32,
    color: HighlightColor,
}

#[derive(Properties, PartialEq)]
struct SquareProps {
    x: i32,
    y: i32,
    color: PieceColor,
    is_highlighted: bool,
}

#[function_component(BoardSquare)]
fn board_square(props: &SquareProps) -> Html {
    let color = if props.color == PieceColor::White {
        "fill:rgb(245,245,245)"
    } else {
        "fill:rgb(176,224,230)"
    };
    let x = format!("{}%", props.x as f32 * 12.5);
    let y = format!("{}%", props.y as f32 * 12.5);
    let width = "12.5%";
    let height = "12.5%";

    html! {
        <>
        <rect class={"svg"} x={x.clone()} y={y.clone()} {width} {height} style={color}/>
        if props.is_highlighted {
            <Highlight x={props.x} y={props.y} color={HighlightColor::Red}/>
        }
        </>
    }
}

#[function_component(Highlight)]
fn highlight(props: &HighlightProps) -> Html {
    let x = format!("{}%", props.x as f32 * 12.5 + 0.5);
    let y = format!("{}%", props.y as f32 * 12.5 + 0.5);
    let width = "11.5%";
    let height = "11.5%";
    let color = match props.color {
        HighlightColor::Red => "red",
        HighlightColor::Yellow => "yellow",
    };
    let style = format!("fill-opacity:0;stroke:{color};stroke-width:3");
    html! {
        <rect class={"svg"} {x} {y} rx={5} ry={5} {width} {height} {style}/>
    }
}

#[derive(Properties, PartialEq)]
struct PieceProps {
    pos: Pos,
    piece: Piece,
}

#[function_component(PieceImage)]
fn piece_image(props: &PieceProps) -> Html {
    let x = format!("{}%", (props.pos.file.as_u8() - 1) as f32 * 12.5);
    let y = format!("{}%", (8 - props.pos.rank.get()) as f32 * 12.5);
    let handles = use_ref(|| Handles::new());
    html! {
        <image class={"svg"} width={"12.5%"} height={"12.5%"} href={handles.get(&props.piece)} {x} {y}/>
    }
}

#[derive(Debug, Clone)]
pub struct Handles(HashMap<Piece, String>);

impl Handles {
    pub fn new() -> Self {
        use PieceType::*;
        let mut inner = HashMap::new();
        for kind in [Pawn, King, Queen, Rook, Bishop, Knight] {
            for color in [PieceColor::White, PieceColor::Black] {
                let prefix = match color {
                    PieceColor::White => "w",
                    PieceColor::Black => "b",
                };
                let postfix = match kind {
                    Pawn => "P",
                    King => "K",
                    Queen => "Q",
                    Rook => "R",
                    Bishop => "B",
                    Knight => "N",
                };
                let path = format!("./resources/{prefix}{postfix}.svg");
                inner.insert(Piece { kind, color }, path);
            }
        }
        Self(inner)
    }

    pub fn get(&self, key: &Piece) -> Option<String> {
        self.0.get(key).cloned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Square {
    Empty,
    Piece(Piece),
}

pub struct Board {
    state: BoardState,
}

#[derive(Debug, PartialEq, Properties)]
pub struct BoardProps {
    pub move_list: crate::Moves,
}

impl Component for Board {
    type Message = Msg;
    type Properties = BoardProps;
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            state: BoardState::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ClickOnSquare(pos) => {
                gloo::console::log!("Click on square: {}", pos.to_string());
                self.state.stop_highlighting();
                if let Some((piece, from)) = self.state.selected_piece.take() {
                    let mv = if piece.kind == PieceType::King {
                        let king_rank = piece.color.king_rank();
                        let king_home = piece.color.king_home();
                        if from == king_home
                            && pos == Pos::new(File::G, king_rank)
                            && self.state.is_castling_possible(CastlingSide::Short)
                        {
                            Move::castling(CastlingSide::Short)
                        } else if from == king_home
                            && pos == Pos::new(File::C, king_rank)
                            && self.state.is_castling_possible(CastlingSide::Long)
                        {
                            Move::castling(CastlingSide::Long)
                        } else {
                            Move::new(from, pos)
                        }
                    } else if piece.kind == PieceType::Pawn
                        && (pos.rank == (!piece.color).king_rank())
                    {
                        Move::new_with_promoted(from, pos, Some(PromotedTo::Queen))
                    } else {
                        Move::new(from, pos)
                    };
                    if self.state.available_moves(piece, from).contains(&mv) {
                        if self.state.is_legal_move(piece, mv) {
                            let san_move = self.state.to_san_move(mv);
                            // check roundtrip
                            let expected = self.state.from_san_move(san_move);
                            assert_eq!(expected, mv);
                            self.state.make_move(mv);
                            ctx.props().move_list.push(san_move);
                            if self.state.is_checkmate(self.state.turn) {
                                self.state.game_result = Some(GameResult::WinByCheckmate {
                                    checkmated_side: self.state.turn,
                                });
                            }
                            if self.state.is_stalemate() {
                                self.state.game_result = Some(GameResult::DrawByStalemate);
                            }
                        }
                    }
                } else if let Square::Piece(piece) = self.state.square_by_pos(pos) {
                    if piece.color == self.state.turn {
                        self.state.selected_piece = Some((piece, pos));
                        self.state.hightlight_legal_moves(piece, pos);
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let row = move |y| {
            (0..8).map(move |x| {
                let color = if (x + y) % 2 == 0 {
                    PieceColor::White
                } else {
                    PieceColor::Black
                };
                let is_highlighted = self.state.is_highlighted(x as usize + y as usize * 8);
                html! { <BoardSquare {color} {x} {y} {is_highlighted}/> }
            })
        };
        let squares = (0..8).map(|y| row(y)).flatten();
        let pieces = self.state.pieces().map(|(piece, pos)| {
            html! {
                <PieceImage {pos} {piece}/>
            }
        });
        let onclick = ctx.link().callback(|event: MouseEvent| {
            let svg: web_sys::Element = event.target_dyn_into().unwrap();
            let rect = svg.get_bounding_client_rect();
            let x = event.offset_x() as f32 / rect.width() as f32;
            let y = event.offset_y() as f32 / rect.height() as f32;
            let pos = cursor_position_to_pos((x, y));
            Msg::ClickOnSquare(pos)
        });
        let active_piece_highlight = self.state.selected_piece.map(|(piece, pos)| {
            let x = (pos.file.as_u8() - 1) as i32;
            let y = (8 - pos.rank.get()) as i32;
            let color = HighlightColor::Yellow;
            html! {
                <Highlight {x} {y} {color} />
            }
        });
        html! {
            <svg {onclick} class={classes!("h-full", "aspect-square")}>
                { for squares }
                { for pieces }
                { for active_piece_highlight }
            </svg>
        }
    }
}

pub fn cursor_position_to_pos((x, y): (f32, f32)) -> Pos {
    let file = (x * 8.0).ceil() as u8;
    let file = File::from_u8(file);
    let rank = 9 - (y * 8.0).ceil() as u8;
    let rank = Rank::new(rank);
    Pos::new(file, rank)
}

use yew::prelude::*;

use crate::board::BoardState;
use crate::moves::san::Move;
use crate::Msg;

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    pub move_list: crate::Moves,
}

#[function_component(MoveList)]
pub fn move_list(props: &Props) -> Html {
    let moves = props.move_list.inner.borrow();
    let rows = moves.chunks(2).enumerate().map(|(i, r)| {
        let first = r.get(0).map(ToString::to_string).unwrap();
        let second = r.get(1).map(ToString::to_string).unwrap_or_default();
        html! {
            <div>
                { format!("{i}. {first} {second}") }
            </div>
        }
    });
    html! {
        <div>
            { for rows }
        </div>
    }
}

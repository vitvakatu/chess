use crate::{moves::PromotedTo, Message};
use iced::{button, Button, Column, Container, Text};

pub struct PromotionMenu {
    queen: button::State,
}

impl PromotionMenu {
    pub fn new() -> Self {
        PromotionMenu {
            queen: Default::default(),
        }
    }

    pub fn view(&mut self) -> Container<'_, Message> {
        let column = Column::new().push(
            Button::new(&mut self.queen, Text::new("Queen"))
                .on_press(Message::ClosePromotionMenu(Some(PromotedTo::Queen))),
        );
        Container::new(column)
    }
}

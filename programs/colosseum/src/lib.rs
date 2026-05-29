#![no_std]

use sails_rs::{cell::RefCell, prelude::*};

mod game;

pub struct ColosseumProgram {
    state: RefCell<game::GameState>,
}

#[program]
impl ColosseumProgram {
    pub fn new() -> Self {
        Self { state: RefCell::new(game::GameState::default()) }
    }

    pub fn game(&self) -> game::GameService<'_> {
        game::GameService::new(&self.state)
    }
}

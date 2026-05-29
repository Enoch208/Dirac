#![no_std]
#![allow(unused_imports)]

use sails_rs::cell::RefCell;
use sails_rs::gstd::msg;
use sails_rs::prelude::*;

mod admin;
mod config;
mod game;
mod state;
mod types;

pub struct DiracProgram {
    state: RefCell<state::GameState>,
}

#[program]
impl DiracProgram {
    pub fn new() -> Self {
        let mut initial = state::GameState::default();
        initial.operator = msg::source();
        Self { state: RefCell::new(initial) }
    }

    pub fn game(&self) -> game::GameService<'_> {
        game::GameService::new(&self.state)
    }

    #[export(route = "admin")]
    pub fn admin(&self) -> admin::AdminService<'_> {
        admin::AdminService::new(&self.state)
    }
}

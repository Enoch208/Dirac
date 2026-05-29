use crate::config::Config;
use crate::state::GameState;
use sails_rs::cell::RefCell;
use sails_rs::gstd::{exec, msg};
use sails_rs::prelude::*;

#[event]
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum AdminEvents {
    RateUpdated { rate: u128, set_at_block: u32 },
    Paused,
    Unpaused,
    PotSeeded { amount: u128, pot: u128 },
    ConfigUpdated,
}

pub struct AdminService<'a> {
    state: &'a RefCell<GameState>,
}

impl<'a> AdminService<'a> {
    pub fn new(state: &'a RefCell<GameState>) -> Self {
        Self { state }
    }
}

#[service(events = AdminEvents)]
impl AdminService<'_> {
    #[export(unwrap_result)]
    pub fn set_vara_usd_rate(&mut self, rate: u128) -> Result<(), String> {
        self.ensure_operator()?;
        if rate == 0 {
            return Err("rate must be positive".into());
        }
        let set_at_block = exec::block_height();
        let mut state = self.state.borrow_mut();
        state.rate.rate = rate;
        state.rate.set_at_block = set_at_block;
        drop(state);
        let _ = self.emit_event(AdminEvents::RateUpdated { rate, set_at_block });
        Ok(())
    }

    #[export(unwrap_result)]
    pub fn pause(&mut self) -> Result<(), String> {
        self.ensure_operator()?;
        self.state.borrow_mut().paused = true;
        let _ = self.emit_event(AdminEvents::Paused);
        Ok(())
    }

    #[export(unwrap_result)]
    pub fn unpause(&mut self) -> Result<(), String> {
        self.ensure_operator()?;
        self.state.borrow_mut().paused = false;
        let _ = self.emit_event(AdminEvents::Unpaused);
        Ok(())
    }

    #[export(unwrap_result)]
    pub fn seed_pot(&mut self) -> Result<(), String> {
        let amount = Syscall::message_value();
        let pot = {
            let mut state = self.state.borrow_mut();
            state.pot = state.pot.saturating_add(amount);
            state.pot
        };
        let _ = self.emit_event(AdminEvents::PotSeeded { amount, pot });
        Ok(())
    }

    #[export(unwrap_result)]
    pub fn set_config(&mut self, config: Config) -> Result<(), String> {
        self.ensure_operator()?;
        self.state.borrow_mut().config = config;
        let _ = self.emit_event(AdminEvents::ConfigUpdated);
        Ok(())
    }
}

impl AdminService<'_> {
    fn ensure_operator(&self) -> Result<(), String> {
        if msg::source() == self.state.borrow().operator {
            return Ok(());
        }
        Err("operator only".into())
    }
}

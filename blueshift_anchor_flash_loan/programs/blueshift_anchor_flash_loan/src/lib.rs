#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use anchor_lang::prelude::*;
declare_id!("22222222222222222222222222222222222222222222");
mod states;
mod instructions;
mod errors;
use instructions::*;

#[program]
pub mod blueshift_anchor_flash_loan {
    use super::*;
    #[instruction(discriminator = 0)]
    pub fn borrow(ctx: Context<Borrow>,borrow_amount:u64) -> Result<()> {
        instructions::borrow::handler(ctx,borrow_amount)
    }
    pub fn repay(ctx: Context<Repay>)-> Result<()>{
        instructions::repay::handler(ctx)
    }
}

#[derive(Accounts)]
pub struct Initialize {}

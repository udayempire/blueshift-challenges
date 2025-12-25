#[allow(unexpected_cfgs)]
#[allow(deprecated)]
use anchor_lang::prelude::*;
mod state;
mod errors;
mod instructions;
use instructions::*;

declare_id!("22222222222222222222222222222222222222222222");
#[program]
pub mod blueshift_anchor_escrow {
    use super::*;
    #[instruction(discriminator = 0)]
    pub fn make(ctx: Context<Make>,seed: u64, recieve: u64, amount: u64) -> Result<()> {
        instructions::make::handler(ctx, seed, recieve, amount)
    }
    pub fn take(ctx: Context<Take>) -> Result<()> {
        instructions::take::handler(ctx)
    }
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        instructions::refund::handler(ctx)
    }
}

#[derive(Accounts)]
pub struct Initialize {}

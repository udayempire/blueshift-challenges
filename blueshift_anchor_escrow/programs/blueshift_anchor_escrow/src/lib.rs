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
    pub fn make(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

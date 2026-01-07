#[allow(unexpected_cfgs)]
#[allow(deprecated)]
use anchor_lang::prelude::*;
declare_id!("22222222222222222222222222222222222222222222");
mod states;
mod instructions;
use instructions::*;

#[program]
pub mod blueshift_anchor_flash_loan {
    use super::*;
    #[instruction(discriminator = 0)]
    pub fn borrow(ctx: Context<Borrow>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

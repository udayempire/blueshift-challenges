use anchor_lang::prelude::*;

declare_id!("8HHrQDb1QJ8vevdcZVs7KMhzA6up2pYDjmYrw6kBz9eZ");

#[program]
pub mod blueshift_anchor_flash_loan {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

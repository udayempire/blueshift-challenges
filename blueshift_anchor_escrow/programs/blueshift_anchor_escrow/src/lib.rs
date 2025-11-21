use anchor_lang::prelude::*;

declare_id!("GsbqNT1jVac397qxsiG2G7Ezp1T54uV6x8Pc7UuA7XJe");

#[program]
pub mod blueshift_anchor_escrow {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

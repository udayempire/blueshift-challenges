use anchor_lang::prelude::*;

#[account(discriminator =1)]
#[derive(InitSpace)]
pub struct LoanState{
    pub borrower: Pubkey,
    pub protocol: Pubkey,
    pub mint: Pubkey,
    pub bump: u8
}

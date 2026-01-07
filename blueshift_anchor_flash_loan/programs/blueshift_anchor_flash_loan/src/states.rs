use anchor_lang::prelude::*;

#[account(discriminator =1)]
#[derive(InitSpace)]
pub struct LoanState{
    borrower: Pubkey,
    protocol: Pubkey,
    mint: Pubkey,
    bump: u8
}

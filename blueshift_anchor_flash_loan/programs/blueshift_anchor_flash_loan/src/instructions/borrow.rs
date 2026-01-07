#![allow(deprecated)]
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use crate::{states::LoanState};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint,TokenAccount}
};

#[derive(Accounts)]
pub struct Loan<'info>{
    #[account(
        mut
    )]
    pub borrower : Signer<'info>, // the user requesting the flash loan.
    #[account(
        init,
        payer = borrower,
        space= Loan::INIT_SPACE + Loan::DISCRIMINATOR.len(),
        seeds= [b"protocol",borrower.key().as_ref()],
        bump
    )]
    pub protocol: Account<'info,LoanState>, //PDA that owns the protocol's liquidity pool.
    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: Account<'info,Mint>, //the specific token being borrowed
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = borrower,
        associated_token::token_program = token_program
    )]
    pub borrower_ata: Account<'info,TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = protocol,
        associated_token::token_program = token_program
    )]
    pub protocol_ata: Account<'info,TokenAccount>,
    pub associated_token_program: Program<'info,AssociatedToken>,
    pub token_program : Program<'info,System>,
    pub system_program: Program<'info,System>
}
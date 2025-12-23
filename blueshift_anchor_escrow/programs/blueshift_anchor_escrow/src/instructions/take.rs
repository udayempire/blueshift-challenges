#![allow(deprecated)]
#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use anchor_spl::{
    token::Mint, 
    token::TokenAccount,
    // associated_token::AssociatedToken,
};
use crate::{state::Escrow};
#[derive(Accounts)]
pub struct Take<'info>{
    //the user that accepts the terms of the maker and is making the exchange
    #[account(mut)]
    pub taker: Signer<'info>,
    // the user that initially set the terms
    #[account(mut)]
    pub maker: SystemAccount<'info>,
    #[account(
        mut,
        close = maker,
        seeds= [b"escrow",maker.key().as_ref(),escrow.seed.to_bytes().as_ref()],
        bump = escrow.bump,
        has_one = maker @ EscrowError::InvalidMaker, //he token that the maker has deposited
        has_one = mint_a @ EscrowError::InvalidMintA,
        has_one = mint_b @ EscrowError::InvalidMintB
    )]
    pub escrow: Box<Account<'info,Escrow>>,
    /// Token Accounts
    pub mint_a: Box<InterfaceAccount<'info,Mint>>, //he token that the maker has deposited
    pub mint_b: Box<InterfaceAccount<'info,Mint>>, //he token that the maker wants om exchange 
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: Box<InterfaceAccount<'info,TokenAccount>>, // the token account associated with the escrow and mint_a that will send the tokens to the taker
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint =mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_a: Box<InterfaceAccount<'info,TokenAccount>>,
    pub taker_ata_b: Box<InterfaceAccount<'info,TokenAccount>>, //, 
}

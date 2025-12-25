#![allow(deprecated)]
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use crate::{state::Escrow, errors::EscrowError};
use anchor_spl::{
    associated_token::{AssociatedToken},
    token_interface::{Mint,TokenAccount},
    token::{
        close_account, transfer_checked, CloseAccount, Token, TransferChecked,
    }, // associated_token::AssociatedToken,
};
#[derive(Accounts)]
pub struct Take<'info> {
    //the user that accepts the terms of the maker and is making the exchange
    #[account(mut)]
    pub taker: Signer<'info>,
    // the user that initially set the terms
    #[account(mut)] 
    pub maker: SystemAccount<'info>,
    #[account(
        mut,
        close = maker,
        seeds= [b"escrow",maker.key().as_ref(),escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
        has_one = maker @ EscrowError::InvalidMaker, //he token that the maker has deposited
        has_one = mint_a @ EscrowError::InvalidMintA,
        has_one = mint_b @ EscrowError::InvalidMintB,
    )]
    pub escrow: Box<Account<'info, Escrow>>,
    /// Token Accounts  
    #[account(
            mint::token_program = token_program
    )]
    pub mint_a: Box<InterfaceAccount<'info, Mint>>, //he token that the maker has deposited
    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: Box<InterfaceAccount<'info, Mint>>, //he token that the maker wants om exchange
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>, // the token account associated with the escrow and mint_a that will send the tokens to the taker
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint =mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>, //he token account associated with the taker and mint_a that will receive the tokens from the vault
    #[account(
        init_if_needed,
        payer= taker,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>, // the token account associated with the taker and mint_b that will send the tokens to the maker
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_b: InterfaceAccount<'info, TokenAccount>, //the token account associated with the maker and mint_b that will receive the tokens to the taker
    pub associated_token_program: Program<'info, AssociatedToken>, // to create token accounts if missing
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}   

impl<'info>Take<'info> {
    fn transfer_to_maker(&mut self) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.taker_ata_b.to_account_info(),
                    to: self.maker_ata_b.to_account_info(),
                    mint: self.mint_b.to_account_info(),
                    authority: self.taker.to_account_info(),
                },
            ),
            self.escrow.receive,
            self.mint_b.decimals,
        )?;
        Ok(())
    }
    fn withdraw_and_close_vault(&mut self) -> Result<()> {
        //create signer seeds for the vault
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_be_bytes()[..],
            &[self.escrow.bump],
        ]];
        //Transfer Token A (Vault -> Taker)
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.vault.to_account_info(),
                    mint: self.mint_a.to_account_info(),
                    to: self.taker_ata_a.to_account_info(),
                    authority: self.escrow.to_account_info(),
                },
                &signer_seeds,
            ),
            self.vault.amount,
            self.mint_a.decimals,
        )?;
        // Close The Vault
        close_account(CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            CloseAccount {
                account: self.vault.to_account_info(),
                authority: self.escrow.to_account_info(),
                destination: self.maker.to_account_info(),
            },
            &signer_seeds,
        ))?;
        Ok(())
    }
}

pub fn handler(ctx: Context<Take>) -> Result<()> {
    // Transfer Token B to Maker
    ctx.accounts.transfer_to_maker()?;
    // withdraw and close the vault
    ctx.accounts.withdraw_and_close_vault()?;
    Ok(())
}
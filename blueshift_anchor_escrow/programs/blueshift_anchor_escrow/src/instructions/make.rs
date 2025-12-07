#![allow(deprecated)]
#![allow(unexpected_cfgs)]
use crate::{errors::EscrowError, state::Escrow};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Mint,
    token::{spl_token::instruction::transfer_checked, TokenAccount, TransferChecked},
};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(
        init,
        payer = maker,
        space = Escrow::INIT_SPACE + Escrow::DISCRIMINATOR.len(),
        seeds = [b"escrow",maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>, //the account holding the exchange terms (maker, mints, amounts)
    //Token accounts
    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>, // the token that the maker is depositing
    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,  //the token that the maker wants in exchange
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>, //the token account associated with the maker and mint_a used to deposit tokens in the vault
    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>, //the token account associated with the escrow and mint_a where deposited tokens are parked
    pub associated_token_program: Program<'info, AssociatedToken>, //the associated token program used to create the associated token accounts
    pub token_program: Program<'info, TokenAccount>, //the token program used to CPI the transfer
    pub system_program: Program<'info, System>, //the system program used to create the Escrow
}

impl<'info>Make<'info> {
    //initialze escrow
    fn populate_escrow(&mut self, seed: u64, amount: u64, bump: u8) -> Result<()> {
        self.escrow.set_inner(Escrow {
            seed,
            maker: self.maker.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            recieve: amount,
            bump,
        });
        Ok(())
    }
    /// # Deposit the tokens
    fn deposit_tokens(&self, amount: u64) -> Result<()> {
        //why using transfer checked?
        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.maker_ata_a.to_account_info(),
                    mint: self.mint_a.to_account_info(),
                    to: self.vault.to_account_info(),
                    authority: self.maker.to_account_info(),
                },
            ),
            amount,
            self.mint_a.decimals,
        )?;
        Ok(())
    }
}

pub fn handler(ctx: Context<Make>, seed: u64, recieve: u64, amount: u64) -> Result<()> {
    //validate the amount
    require_gt!(recieve, 0, EscrowError::InvalidAmount);
    require_gt!(amount, 0, EscrowError::InvalidAmount);
    //save the escrow data
    ctx.accounts
        .populate_escrow(seed, recieve, ctx.bumps.escrow);
    //deposit Tokens
    ctx.accounts.deposit_tokens(amount)?;
    Ok(())
}

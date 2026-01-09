#![allow(deprecated)]
#![allow(unexpected_cfgs)]
use crate::states::LoanState;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, TokenAccount, Transfer},
};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Loan<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>, // the user requesting the flash loan.
    #[account(
        init,
        payer = borrower,
        space= LoanState::INIT_SPACE + LoanState::DISCRIMINATOR.len(),
        seeds= [b"protocol"],
        bump
    )]
    pub protocol: Account<'info, LoanState>, //PDA that owns the protocol's liquidity pool.
    #[account(
        mint::token_program = token_program
    )]
    pub mint: Account<'info, Mint>, //the specific token being borrowed
    #[account(
        init_if_needed,
        payer = borrower,
        associated_token::mint = mint,
        associated_token::authority = borrower,
        associated_token::token_program = token_program
    )]
    pub borrower_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = protocol,
        associated_token::token_program = token_program
    )]
    pub protocol_ata: Account<'info, TokenAccount>,
    #[account(
        address = INSTRUCTIONS_SYSVAR_ID
    )]
    // CHECK: InstructionsSysvar account
    instructions: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Loan<'info> {
    fn transfer_from_protocol(&mut self, borrow_amount: u64, bump: u8) -> Result<()> {
        // Making sure we are not sending invalid amount
        require!(borrow_amount > 0, ProtocolError::InvalidAmount);
        // derive signer seeds for protocol amount
        let signer_seeds: &[&[&[u8]]] = &[&[b"protocol".as_ref(), &[self.protocol.bump]]];
        //transfer funds from protocol to borrower
        transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.protocol_ata.to_account_info(),
                    to: self.borrower_ata.to_account_info(),
                    authority: self.protocol.to_account_info(),
                },
                signer_seeds,
            ),
            borrow_amount,
        );
        Ok(())
    }
}

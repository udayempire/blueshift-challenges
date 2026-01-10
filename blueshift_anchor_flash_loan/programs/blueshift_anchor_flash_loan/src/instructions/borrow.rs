#![allow(deprecated)]
#![allow(unexpected_cfgs)]
use crate::instruction::Repay;
use crate::{errors::ProtocolError, ID};
use anchor_lang::{
    prelude::*,
    solana_program::sysvar::instructions::{
        load_current_index_checked, load_instruction_at_checked, ID as INSTRUCTIONS_SYSVAR_ID,
    },
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>, // the user requesting the flash loan.
    #[account(
        seeds= [b"protocol"],
        bump
    )]
    /*we use system account because we want protcol to only signning authority and not to hold data so we dont need any SPACE.
    Also saves a init program for protocol as its derived on-the-fly each time.
    */
    pub protocol: SystemAccount<'info>, //PDA that owns the protocol's liquidity pool.
    #[account()]
    pub mint: Account<'info, Mint>, //the specific token being borrowed
    ///3 (line 97)
    #[account(
        init_if_needed,
        payer = borrower,
        associated_token::mint = mint,
        associated_token::authority = borrower,
        associated_token::token_program = token_program
    )]
    pub borrower_ata: Account<'info, TokenAccount>,
    ///4
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = protocol,
        associated_token::token_program = token_program
    )]
    pub protocol_ata: Account<'info, TokenAccount>,
    /// CHECK: Instructions sysvar account - validated by address constraint
    #[account(address = INSTRUCTIONS_SYSVAR_ID)]
    pub instructions: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Borrow<'info> {
    fn transfer_from_protocol(&mut self, borrow_amount: u64, bump: u8) -> Result<()> {
        //Instruction Introspection is the primary means by which we secure our program
        let ixs = self.instructions.to_account_info();
        //checking the position of this instruction making sure its 1st
        let current_index = load_current_index_checked(&self.instructions)?;
        //The borrow must be instruction #0,so borrower can't do anything sneaky before borrowing.
        require_eq!(current_index, 0, ProtocolError::InvalidIx);
        //check the number of instruction in this transaction
        let instruction_sysvar = ixs.try_borrow_data()?;
        // 0-2 because first 2 elements of array contains the number of instruction (2,0)(means 2 instruction ) 2 bytes
        let len = u16::from_le_bytes(instruction_sysvar[0..2].try_into().unwrap());
        //ensure repay instruction exist
        //Solana helper function that reads an instruction from the Instructions Sysvar.(load_instruction_at_checked)
        //Returns a struct with: program_id, accounts, and data.
        if let Ok(repay_ix) = load_instruction_at_checked(len as usize - 1, &ixs) {
            //Instruction checks
            require_keys_eq!(repay_ix.program_id, ID, ProtocolError::InvalidProgram);
            require!(
                repay_ix.data[0..8].eq(Repay::DISCRIMINATOR),
                ProtocolError::InvalidIx
            );
            require_keys_eq!(
                repay_ix
                    .accounts
                    .get(3)
                    .ok_or(ProtocolError::InvalidBorrowerAta)?
                    .pubkey,
                self.borrower_ata.key(),
                ProtocolError::InvalidBorrowerAta
            );
            require_keys_eq!(
                repay_ix
                    .accounts
                    .get(4)
                    .ok_or(ProtocolError::InvalidProtocolAta)?
                    .pubkey,
                self.protocol_ata.key(),
                ProtocolError::InvalidProtocolAta
            );
        } else {
            return Err(ProtocolError::MissingRepayIx.into());
        }
        // Making sure we are not sending invalid amount
        require!(borrow_amount > 0, ProtocolError::InvalidAmount);
        // all check done
        // derive signer seeds for protocol amount
        let signer_seeds: &[&[&[u8]]] = &[&[b"protocol".as_ref(), &[bump]]];
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
        )?;

        Ok(())
    }
}

pub fn handler(ctx: Context<Borrow>, borrow_amount: u64) -> Result<()> {
    ctx.accounts
        .transfer_from_protocol(borrow_amount, ctx.bumps.protocol)?;
    Ok(())
}

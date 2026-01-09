#![allow(deprecated)]
#![allow(unexpected_cfgs)]
use crate::{errors::ProtocolError,ID};
use anchor_lang::{
    prelude::*,
    solana_program::sysvar::instructions::{
        load_current_index_checked, load_instruction_at_checked,
    },
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, TokenAccount, Transfer,Token},
};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Loan<'info> {
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
        associated_token::mint = mint,
        associated_token::authority = protocol,
        associated_token::token_program = token_program
    )]
    pub protocol_ata: Account<'info, TokenAccount>,
    #[account(
        address = INSTRUCTIONS_SYSVAR_ID
    )]
    // CHECK: InstructionsSysvar account
    instructions: UncheckedAccount<'info>, //contains all instructions in this transaction
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Loan<'info> {
    fn transfer_from_protocol(&mut self, borrow_amount: u64, bump: u8) -> Result<()> {
        // Making sure we are not sending invalid amount
        require!(borrow_amount > 0, ProtocolError::InvalidAmount);
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
        //Instruction Introspection is the primary means by which we secure our program
        let ixs = self.instructions.to_account_info();
        //checking the position of this instruction making sure its 1st
        let current_index = load_current_index_checked(&self.instructions)?;
        //The borrow must be instruction #0,so borrower can't do anything sneaky before borrowing.
        require_eq!(current_index, 0, ProtocolError::InvalidIx);
        //check the number of instruction in this transaction
        let instruction_sysvar = ixs.try_borrow_data()?;
        let len = u16::from_le_bytes(instruction_sysvar[0..2].try_into().unwrap());
        //ensure repay instruction exist
        //Solana helper function that reads an instruction from the Instructions Sysvar.(load_instruction_at_checked)
        //Returns a struct with: program_id, accounts, and data.
        if let Ok(repay_ix) = load_instruction_at_checked(len as usize - 1, &ixs) {
            //Instruction checks
            require_keys_eq!(repay_ix.program_id, ID, ProtocolError::InvalidProgram);
            require!(
                repay_ix.data[0..8].eq(instruction::Repay::DISCRIMINATOR),
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
        }else{
            return Err(ProtocolError::MissingRepayIx.into())
        }
        Ok(())
    }
}

pub fn handler(ctx: Context<Loan>,borrow_amount:u64)->Result<()>{
    ctx.accounts.transfer_from_protocol(borrow_amount, ctx.bumps.protocol)?;
    Ok(())
}
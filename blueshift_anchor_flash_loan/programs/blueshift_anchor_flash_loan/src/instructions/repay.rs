#![allow(deprecated)]
#![allow(unexpected_cfgs)]
use anchor_lang::{prelude::*, solana_program::sysvar::instructions::{load_instruction_at_checked}};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, TokenAccount,Token, Transfer,transfer}
};

use crate::errors::ProtocolError;

// use account introspection to retrieve the amount_borrowed from borrower instruction data 
// calculate fee and transfer the borrowed amount back to the protocol

#[derive(Accounts)]
pub struct Repay<'info>{
    #[account(
        mut
    )]
    pub borrower: Signer<'info>,
    #[account(
        seeds=[b"protocol"],
        bump
    )]
    pub protocol: SystemAccount<'info>,
    #[account(
        mint::token_program= token_program
    )]
    pub mint: Account<'info,Mint>, 
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = borrower,
        associated_token::token_program = token_program
    )]
    pub borrower_ata: Account<'info,TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = protocol,
        associated_token::token_program = token_program
    )]
    pub protocol_ata: Account<'info,TokenAccount>,
    //Instruction sysvar for introspection
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info,AssociatedToken>,
    pub token_program: Program<'info,Token>,
    pub system_program: Program<'info,System>
}

impl <'info>Repay<'info>{
    fn refund_to_protocol(&mut self)->Result<()>{
        let ixs = self.instructions.to_account_info();
        let mut amount_borrowed: u64;
        if let Ok(borrow_ixs)= load_instruction_at_checked(0,&ixs){
            //check amount borrowed
            let mut borrowed_data: [u8;8] = [0u8;8];
            borrowed_data.copy_from_slice(&borrow_ixs.data[8..16]);
            amount_borrowed = u64::from_le_bytes(borrowed_data);
        }else{
            return Err(ProtocolError::MissingBorrowIx.into());
        }
        // add the fee amount borrowed(hardcoded to 500 basis point here)
        let fee = (amount_borrowed as u128).checked_mul(500).unwrap().checked_div(10_000).ok_or(ProtocolError::Overflow)? as u64;
        amount_borrowed = amount_borrowed.checked_add(fee).ok_or(ProtocolError::Overflow)?;
        // Transfer the funds from protocol to borrower
        transfer(
            CpiContext::new(
                self.token_program.to_account_info(), 
                Transfer{
                    from: self.borrower_ata.to_account_info(),
                    to: self.protocol_ata.to_account_info(),
                    authority: self.borrower.to_account_info()
                }
            ),amount_borrowed
        )?;
        Ok(())
    }
}

pub fn handler(ctx:Context<Repay>)->Result<()>{
    ctx.accounts.refund_to_protocol()?;
    Ok(())
}
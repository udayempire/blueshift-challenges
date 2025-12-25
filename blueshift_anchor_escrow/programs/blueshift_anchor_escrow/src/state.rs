use anchor_lang::prelude::*;

#[account(discriminator =1)]
#[derive(InitSpace)]
pub struct Escrow{
    pub seed: u64, 
    pub maker: Pubkey, //the user that decides the terms and deposits the mint_a into the Escrow
    pub mint_a: Pubkey, //the token that the maker is depositing
    pub mint_b: Pubkey, //the token that the maker wants in exchange
    pub receive: u64,
    pub bump: u8
}

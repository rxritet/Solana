use anchor_lang::prelude::*;

declare_id!("FTn5SYqQXK2MSKoC8Uzt9BmFCJy5UbHEcJVqNBNxeUJH");

#[program]
pub mod first_solana_program {
    use super::*;

    pub fn say_hello(_ctx: Context<SayHello>) -> Result<()> {
        msg!("Hello from my first Solana program!");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SayHello {}
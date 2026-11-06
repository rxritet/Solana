use anchor_lang::prelude::*;

pub mod constants {
    pub const COUNTER_SEED: &[u8] = b"counter";
}

pub mod state {
    use super::*;

    #[account]
    pub struct Counter {
        pub authority: Pubkey,
        pub count: u64,
    }
}

use constants::COUNTER_SEED;
use state::Counter;

declare_id!("EmnHY9cx5uVGQZnj3T3sRqHYBEaUD3ZpiSMN4D6FHcnu");

#[program]
pub mod counter_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.authority = ctx.accounts.payer.key();
        counter.count = 0;

        msg!("Counter created. Current value: {}", counter.count);
        Ok(())
    }

    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count += 1;

        msg!("Counter incremented. Current value: {}", counter.count);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [COUNTER_SEED],
        bump,
        space = 8 + 32 + 8
    )]
    pub counter: Account<'info, Counter>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(
        mut,
        seeds = [COUNTER_SEED],
        bump,
        has_one = authority
    )]
    pub counter: Account<'info, Counter>,

    pub authority: Signer<'info>,
}
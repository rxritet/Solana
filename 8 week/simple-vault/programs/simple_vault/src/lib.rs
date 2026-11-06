use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

declare_id!("EbojNUfh9Jk6dyZaaQAJWofbdnkvdxfQbeAPq6iWoHAu");

#[program]
pub mod simple_vault {
    use super::*;

    /// Creates the vault state and the PDAs that hold deposited tokens and
    /// represent vault shares for one SPL token mint.
    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault_state;
        vault.admin = ctx.accounts.admin.key();
        vault.token_mint = ctx.accounts.token_mint.key();
        vault.vault_token_account = ctx.accounts.vault_token_account.key();
        vault.share_mint = ctx.accounts.share_mint.key();
        vault.total_tokens = 0;
        vault.total_shares = 0;
        vault.bump = ctx.bumps.vault_state;
        Ok(())
    }

    /// Creates exactly one personal card for a user in this vault.
    pub fn create_user_card(ctx: Context<CreateUserCard>) -> Result<()> {
        let card = &mut ctx.accounts.user_card;
        card.vault = ctx.accounts.vault_state.key();
        card.owner = ctx.accounts.user.key();
        card.deposited = 0;
        card.shares = 0;
        card.bump = ctx.bumps.user_card;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = admin,
        space = VaultState::LEN,
        seeds = [b"vault", token_mint.key().as_ref()],
        bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        init,
        payer = admin,
        token::mint = token_mint,
        token::authority = vault_state,
        seeds = [b"vault_tokens", token_mint.key().as_ref()],
        bump,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = vault_state,
        seeds = [b"share_mint", token_mint.key().as_ref()],
        bump,
    )]
    pub share_mint: Account<'info, Mint>,

    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateUserCard<'info> {
    #[account(
        seeds = [b"vault", vault_state.token_mint.as_ref()],
        bump = vault_state.bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        init,
        payer = user,
        space = UserCard::LEN,
        seeds = [b"user_card", vault_state.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub user_card: Account<'info, UserCard>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct VaultState {
    pub admin: Pubkey,
    pub token_mint: Pubkey,
    pub vault_token_account: Pubkey,
    pub share_mint: Pubkey,
    pub total_tokens: u64,
    pub total_shares: u64,
    pub bump: u8,
}

impl VaultState {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 32 + 8 + 8 + 1;
}

#[account]
pub struct UserCard {
    pub vault: Pubkey,
    pub owner: Pubkey,
    pub deposited: u64,
    pub shares: u64,
    pub bump: u8,
}

impl UserCard {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 8 + 1;
}

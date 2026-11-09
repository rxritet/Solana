use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};

declare_id!("EbojNUfh9Jk6dyZaaQAJWofbdnkvdxfQbeAPq6iWoHAu");

#[program]
pub mod simple_vault {
    use super::*;

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

    pub fn create_user_card(ctx: Context<CreateUserCard>) -> Result<()> {
        let card = &mut ctx.accounts.user_card;
        card.vault = ctx.accounts.vault_state.key();
        card.owner = ctx.accounts.user.key();
        card.deposited = 0;
        card.bump = ctx.bumps.user_card;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::ZeroAmount);

        let vault = &ctx.accounts.vault_state;
        let shares = if vault.total_shares == 0 || vault.total_tokens == 0 {
            amount
        } else {
            (amount as u128)
                .checked_mul(vault.total_shares as u128)
                .and_then(|value| value.checked_div(vault.total_tokens as u128))
                .and_then(|value| u64::try_from(value).ok())
                .ok_or(VaultError::MathOverflow)?
        };
        require!(shares > 0, VaultError::SharesTooSmall);

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.vault_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        let token_mint = ctx.accounts.vault_state.token_mint;
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault",
            token_mint.as_ref(),
            &[ctx.accounts.vault_state.bump],
        ]];
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.share_mint.to_account_info(),
                    to: ctx.accounts.user_share_account.to_account_info(),
                    authority: ctx.accounts.vault_state.to_account_info(),
                },
                signer_seeds,
            ),
            shares,
        )?;

        let vault = &mut ctx.accounts.vault_state;
        vault.total_tokens = vault
            .total_tokens
            .checked_add(amount)
            .ok_or(VaultError::MathOverflow)?;
        vault.total_shares = vault
            .total_shares
            .checked_add(shares)
            .ok_or(VaultError::MathOverflow)?;
        let card = &mut ctx.accounts.user_card;
        card.deposited = card
            .deposited
            .checked_add(amount)
            .ok_or(VaultError::MathOverflow)?;
        card.shares = card
            .shares
            .checked_add(shares)
            .ok_or(VaultError::MathOverflow)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, shares_amount: u64) -> Result<()> {
        require!(shares_amount > 0, VaultError::ZeroAmount);

        let vault = &ctx.accounts.vault_state;
        require!(vault.total_shares > 0, VaultError::VaultEmpty);
        let tokens_out = (shares_amount as u128)
            .checked_mul(vault.total_tokens as u128)
            .and_then(|value| value.checked_div(vault.total_shares as u128))
            .and_then(|value| u64::try_from(value).ok())
            .ok_or(VaultError::MathOverflow)?;
        require!(tokens_out > 0, VaultError::WithdrawTooSmall);

        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.share_mint.to_account_info(),
                    from: ctx.accounts.user_share_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            shares_amount,
        )?;

        let token_mint = ctx.accounts.vault_state.token_mint;
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault",
            token_mint.as_ref(),
            &[ctx.accounts.vault_state.bump],
        ]];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.vault_state.to_account_info(),
                },
                signer_seeds,
            ),
            tokens_out,
        )?;

        let vault = &mut ctx.accounts.vault_state;
        vault.total_tokens = vault
            .total_tokens
            .checked_sub(tokens_out)
            .ok_or(VaultError::MathOverflow)?;
        vault.total_shares = vault
            .total_shares
            .checked_sub(shares_amount)
            .ok_or(VaultError::MathOverflow)?;
        let card = &mut ctx.accounts.user_card;
        card.deposited = card.deposited.saturating_sub(tokens_out);
        card.shares = card.shares.saturating_sub(shares_amount);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(init, payer = admin, space = VaultState::LEN, seeds = [b"vault", token_mint.key().as_ref()], bump)]
    pub vault_state: Account<'info, VaultState>,
    #[account(init, payer = admin, token::mint = token_mint, token::authority = vault_state, seeds = [b"vault_tokens", token_mint.key().as_ref()], bump)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(init, payer = admin, mint::decimals = 6, mint::authority = vault_state, seeds = [b"share_mint", token_mint.key().as_ref()], bump)]
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
    #[account(seeds = [b"vault", vault_state.token_mint.as_ref()], bump = vault_state.bump)]
    pub vault_state: Account<'info, VaultState>,
    #[account(init, payer = user, space = UserCard::LEN, seeds = [b"user_card", vault_state.key().as_ref(), user.key().as_ref()], bump)]
    pub user_card: Account<'info, UserCard>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"vault", vault_state.token_mint.as_ref()], bump = vault_state.bump)]
    pub vault_state: Account<'info, VaultState>,
    #[account(mut, seeds = [b"user_card", vault_state.key().as_ref(), user.key().as_ref()], bump = user_card.bump)]
    pub user_card: Account<'info, UserCard>,
    #[account(mut, seeds = [b"vault_tokens", vault_state.token_mint.as_ref()], bump, token::mint = vault_state.token_mint, token::authority = vault_state)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"share_mint", vault_state.token_mint.as_ref()], bump, mint::authority = vault_state)]
    pub share_mint: Account<'info, Mint>,
    #[account(mut, token::mint = vault_state.token_mint, token::authority = user)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = share_mint, token::authority = user)]
    pub user_share_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, seeds = [b"vault", vault_state.token_mint.as_ref()], bump = vault_state.bump)]
    pub vault_state: Account<'info, VaultState>,
    #[account(mut, seeds = [b"user_card", vault_state.key().as_ref(), user.key().as_ref()], bump = user_card.bump)]
    pub user_card: Account<'info, UserCard>,
    #[account(mut, seeds = [b"vault_tokens", vault_state.token_mint.as_ref()], bump, token::mint = vault_state.token_mint, token::authority = vault_state)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"share_mint", vault_state.token_mint.as_ref()], bump, mint::authority = vault_state)]
    pub share_mint: Account<'info, Mint>,
    #[account(mut, token::mint = vault_state.token_mint, token::authority = user)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = share_mint, token::authority = user)]
    pub user_share_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
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

#[error_code]
pub enum VaultError {
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Shares would be zero - deposit too small")]
    SharesTooSmall,
    #[msg("Vault is empty")]
    VaultEmpty,
    #[msg("Withdraw amount too small")]
    WithdrawTooSmall,
    #[msg("Math overflow")]
    MathOverflow,
}

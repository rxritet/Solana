use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount, TransferChecked},
};

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
        vault.paused = false;
        vault.total_deposited = 0;
        vault.bump = ctx.bumps.vault_state;
        Ok(())
    }

    pub fn create_user_card(ctx: Context<CreateUserCard>) -> Result<()> {
        require!(!ctx.accounts.vault_state.paused, VaultError::VaultPaused);
        let card = &mut ctx.accounts.user_card;
        card.vault = ctx.accounts.vault_state.key();
        card.owner = ctx.accounts.user.key();
        card.deposited = 0;
        card.bump = ctx.bumps.user_card;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        require!(!ctx.accounts.vault_state.paused, VaultError::VaultPaused);
        require!(amount > 0, VaultError::ZeroAmount);

        let shares = shares_for_deposit(
            amount,
            ctx.accounts.share_mint.supply,
            ctx.accounts.vault_token_account.amount,
        )?;
        require!(shares > 0, VaultError::ZeroShares);

        token::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    mint: ctx.accounts.token_mint.to_account_info(),
                    to: ctx.accounts.vault_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
            ctx.accounts.token_mint.decimals,
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
        vault.total_deposited = vault.total_deposited.checked_add(amount).ok_or(VaultError::MathOverflow)?;
        let card = &mut ctx.accounts.user_card;
        card.deposited = card.deposited.checked_add(amount).ok_or(VaultError::MathOverflow)?;
        Ok(())
    }

    pub fn set_paused(ctx: Context<SetPaused>, paused: bool) -> Result<()> {
        ctx.accounts.vault_state.paused = paused;
        Ok(())
    }
}

pub fn shares_for_deposit(amount: u64, share_supply: u64, vault_balance: u64) -> Result<u64> {
    if share_supply == 0 || vault_balance == 0 {
        return Ok(amount);
    }
    let shares = (amount as u128)
        .checked_mul(share_supply as u128)
        .ok_or(VaultError::MathOverflow)?
        .checked_div(vault_balance as u128)
        .ok_or(VaultError::MathOverflow)?;
    u64::try_from(shares).map_err(|_| error!(VaultError::MathOverflow))
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
    #[account(mut, seeds = [b"vault", vault_state.token_mint.as_ref()], bump = vault_state.bump, has_one = token_mint, has_one = vault_token_account, has_one = share_mint)]
    pub vault_state: Account<'info, VaultState>,
    #[account(mut, seeds = [b"user_card", vault_state.key().as_ref(), user.key().as_ref()], bump = user_card.bump)]
    pub user_card: Account<'info, UserCard>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut, token::mint = token_mint, token::authority = vault_state)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(mut, mint::authority = vault_state)]
    pub share_mint: Account<'info, Mint>,
    #[account(mut, token::mint = token_mint, token::authority = user)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(init_if_needed, payer = user, associated_token::mint = share_mint, associated_token::authority = user)]
    pub user_share_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetPaused<'info> {
    #[account(mut, has_one = admin)]
    pub vault_state: Account<'info, VaultState>,
    pub admin: Signer<'info>,
}

#[account]
pub struct VaultState {
    pub admin: Pubkey,
    pub token_mint: Pubkey,
    pub vault_token_account: Pubkey,
    pub share_mint: Pubkey,
    pub bump: u8,
    pub paused: bool,
    pub total_deposited: u64,
}
impl VaultState { pub const LEN: usize = 8 + 32 + 32 + 32 + 32 + 1 + 1 + 8; }

#[account]
pub struct UserCard {
    pub vault: Pubkey,
    pub owner: Pubkey,
    pub deposited: u64,
    pub bump: u8,
}
impl UserCard { pub const LEN: usize = 8 + 32 + 32 + 8 + 1; }

#[error_code]
pub enum VaultError {
    #[msg("The vault is paused")]
    VaultPaused,
    #[msg("Deposit amount must be greater than zero")]
    ZeroAmount,
    #[msg("Deposit would mint zero shares")]
    ZeroShares,
    #[msg("Arithmetic overflow")]
    MathOverflow,
}

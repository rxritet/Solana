use anchor_lang::prelude::*;

pub mod constants {
    pub const PROFILE_SEED: &[u8] = b"profile";
    pub const NAME_MAX_LEN: usize = 32;
    pub const BIO_MAX_LEN: usize = 160;
}

pub mod state {
    use super::*;

    #[account]
    pub struct UserProfile {
        pub authority: Pubkey,
        pub name: String,
        pub bio: String,
        pub bump: u8,
    }

    impl UserProfile {
        pub const SPACE: usize = 8 + 32 + 4 + 32 + 4 + 160 + 1;
    }
}

use constants::{BIO_MAX_LEN, NAME_MAX_LEN, PROFILE_SEED};
use state::UserProfile;

declare_id!("WuJdhVCp71LsNuePY812xooTY4r28ogJ9ee9EnLNN8s");

#[program]
pub mod onchain_profile {
    use super::*;

    pub fn create_profile(
        ctx: Context<CreateProfile>,
        name: String,
        bio: String,
    ) -> Result<()> {
        require!(
            name.as_bytes().len() <= NAME_MAX_LEN,
            ProfileError::NameTooLong
        );
        require!(
            bio.as_bytes().len() <= BIO_MAX_LEN,
            ProfileError::BioTooLong
        );

        let profile = &mut ctx.accounts.profile;
        profile.authority = ctx.accounts.authority.key();
        profile.name = name;
        profile.bio = bio;
        profile.bump = ctx.bumps.profile;

        msg!("Profile created for {}", profile.authority);
        Ok(())
    }

    pub fn update_profile(
        ctx: Context<UpdateProfile>,
        name: String,
        bio: String,
    ) -> Result<()> {
        require!(
            name.as_bytes().len() <= NAME_MAX_LEN,
            ProfileError::NameTooLong
        );
        require!(
            bio.as_bytes().len() <= BIO_MAX_LEN,
            ProfileError::BioTooLong
        );

        let profile = &mut ctx.accounts.profile;
        profile.name = name;
        profile.bio = bio;

        msg!("Profile updated");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateProfile<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = UserProfile::SPACE,
        seeds = [PROFILE_SEED, authority.key().as_ref()],
        bump
    )]
    pub profile: Account<'info, UserProfile>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateProfile<'info> {
    #[account(
        mut,
        seeds = [PROFILE_SEED, authority.key().as_ref()],
        bump = profile.bump,
        has_one = authority
    )]
    pub profile: Account<'info, UserProfile>,

    pub authority: Signer<'info>,
}

#[error_code]
pub enum ProfileError {
    #[msg("Имя не должно быть длиннее 32 байт.")]
    NameTooLong,

    #[msg("Описание не должно быть длиннее 160 байт.")]
    BioTooLong,
}

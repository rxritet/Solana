# Simple Vault — week 10

This Anchor project implements deposits and withdrawals backed by SPL share tokens.

## Deposit flow

1. The first deposit mints shares 1:1. Later deposits use
   `amount * total_shares / total_tokens`, calculated in `u128` and rounded down.
2. The user signs a token transfer to move SPL tokens into the vault token account.
3. The vault PDA signs `mint_to` to mint shares into the user's associated token account.
4. `VaultState.total_tokens`, `VaultState.total_shares`, and the user's card are updated safely.

## Withdraw flow

1. `withdraw(shares_amount)` calculates `shares_amount * total_tokens / total_shares` in `u128`.
2. The user signs `burn` to destroy the shares being redeemed.
3. The vault PDA signs a token transfer from the vault account back to the user's token account.
4. Vault totals and the user's card are reduced safely.

## Protections

- Deposit and withdrawal amounts must be positive and result in at least one token/share.
- The depositor must own the asset token account and already have a user-card PDA.
- The vault state verifies its configured token mint, token account, and share mint.
- Withdrawals require a non-empty vault and sufficient shares in the user's share account.

## Build and test

```bash
yarn install
anchor keys sync
anchor build
anchor test --provider.cluster devnet --skip-local-validator --skip-deploy
```

Generated keys, build output, dependencies, and ZIP archives are excluded from Git.

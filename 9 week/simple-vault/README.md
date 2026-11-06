# Simple Vault — week 9

This Anchor project extends the vault foundation with `deposit(amount)` and share tokens.

## Deposit flow

1. The first deposit mints shares 1:1. Later deposits use
   `amount * share_supply / vault_token_balance`, calculated in `u128` and rounded down.
2. The user signs `transfer_checked` to move SPL tokens into the vault token account.
3. The vault PDA signs `mint_to` to mint shares into the user's associated token account.
4. `VaultState.total_deposited` and `UserCard.deposited` are updated safely.

## Protections

- Amount must be positive and result in at least one share.
- The depositor must own the asset token account and already have a user-card PDA.
- The vault state verifies its configured token mint, token account, and share mint.
- The admin can pause card creation and deposits.

## Build and test

```bash
yarn install
anchor keys sync
anchor build
anchor test --provider.cluster devnet --skip-local-validator --skip-deploy
```

Generated keys, build output, dependencies, and ZIP archives are excluded from Git.

# Simple Vault — week 8

Anchor program for the eighth-week assignment. It implements only the foundation:

- `initialize_vault` creates a `VaultState` PDA, a vault SPL-token account and a share mint;
- `create_user_card` creates one `UserCard` PDA per `(vault, user)` pair.

The PDA seeds are:

| Account | Seeds |
| --- | --- |
| Vault state | `vault`, token mint |
| Vault token account | `vault_tokens`, token mint |
| Share mint | `share_mint`, token mint |
| User card | `user_card`, vault state, user |

`UserCard` starts with zero deposited tokens and zero shares. Deposit/withdraw instructions are intentionally out of scope: they are planned for the next weeks.

## Local build

```bash
yarn install
anchor keys sync
anchor build
```

`anchor keys sync` updates the public program ID after Anchor generates the local deploy keypair. The generated `target/` directory and all local keypairs are ignored by Git.

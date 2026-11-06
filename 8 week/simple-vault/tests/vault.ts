import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { createMint, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { SYSVAR_RENT_PUBKEY, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";
import { SimpleVault } from "../target/types/simple_vault";

describe("simple_vault", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SimpleVault as Program<SimpleVault>;
  const admin = (provider.wallet as anchor.Wallet).payer;
  let tokenMint: anchor.web3.PublicKey;
  let vaultState: anchor.web3.PublicKey;
  let vaultTokenAccount: anchor.web3.PublicKey;
  let shareMint: anchor.web3.PublicKey;

  before(async () => {
    tokenMint = await createMint(
      provider.connection,
      admin,
      admin.publicKey,
      null,
      6,
    );

    [vaultState] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), tokenMint.toBuffer()],
      program.programId,
    );
    [vaultTokenAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_tokens"), tokenMint.toBuffer()],
      program.programId,
    );
    [shareMint] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("share_mint"), tokenMint.toBuffer()],
      program.programId,
    );
  });

  it("creates a vault and its token PDAs", async () => {
    await program.methods
      .initializeVault()
      .accounts({
        vaultState,
        vaultTokenAccount,
        shareMint,
        tokenMint,
        admin: admin.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      } as any)
      .rpc();

    const vault = await program.account.vaultState.fetch(vaultState);
    assert.equal(vault.admin.toBase58(), admin.publicKey.toBase58());
    assert.equal(vault.tokenMint.toBase58(), tokenMint.toBase58());
    assert.equal(vault.vaultTokenAccount.toBase58(), vaultTokenAccount.toBase58());
    assert.equal(vault.shareMint.toBase58(), shareMint.toBase58());
    assert.equal(vault.totalTokens.toString(), "0");
    assert.equal(vault.totalShares.toString(), "0");
  });

  it("creates one user card and rejects a duplicate", async () => {
    const [userCard] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user_card"), vaultState.toBuffer(), admin.publicKey.toBuffer()],
      program.programId,
    );

    const accounts = {
      vaultState,
      userCard,
      user: admin.publicKey,
      systemProgram: SystemProgram.programId,
    };
    await program.methods.createUserCard().accounts(accounts as any).rpc();

    const card = await program.account.userCard.fetch(userCard);
    assert.equal(card.vault.toBase58(), vaultState.toBase58());
    assert.equal(card.owner.toBase58(), admin.publicKey.toBase58());
    assert.equal(card.deposited.toString(), "0");
    assert.equal(card.shares.toString(), "0");

    let duplicateWasRejected = false;
    try {
      await program.methods.createUserCard().accounts(accounts as any).rpc();
    } catch {
      duplicateWasRejected = true;
    }
    assert.isTrue(duplicateWasRejected, "a user must have only one card per vault");
  });
});

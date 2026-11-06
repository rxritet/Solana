import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { createMint, getAccount, getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID, transfer } from "@solana/spl-token";
import { SYSVAR_RENT_PUBKEY, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";
import { SimpleVault } from "../target/types/simple_vault";

describe("simple_vault week 9", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.SimpleVault as Program<SimpleVault>;
  const admin = (provider.wallet as anchor.Wallet).payer;
  const UNIT = 1_000_000;
  const units = (value: number) => new BN(value * UNIT);
  let tokenMint: anchor.web3.PublicKey;
  let vaultState: anchor.web3.PublicKey;
  let vaultTokenAccount: anchor.web3.PublicKey;
  let shareMint: anchor.web3.PublicKey;
  let userTokenAccount: anchor.web3.PublicKey;

  const userCard = () => anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user_card"), vaultState.toBuffer(), admin.publicKey.toBuffer()], program.programId,
  )[0];
  const deposit = (amount: BN) => program.methods.deposit(amount).accounts({
    vaultState, userCard: userCard(), tokenMint, vaultTokenAccount, shareMint, userTokenAccount, user: admin.publicKey,
  } as any).rpc();

  before(async () => {
    tokenMint = await createMint(provider.connection, admin, admin.publicKey, null, 6);
    [vaultState] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("vault"), tokenMint.toBuffer()], program.programId);
    [vaultTokenAccount] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("vault_tokens"), tokenMint.toBuffer()], program.programId);
    [shareMint] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("share_mint"), tokenMint.toBuffer()], program.programId);
    userTokenAccount = (await getOrCreateAssociatedTokenAccount(provider.connection, admin, tokenMint, admin.publicKey)).address;
    await program.methods.initializeVault().accounts({
      vaultState, vaultTokenAccount, shareMint, tokenMint, admin: admin.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID, systemProgram: SystemProgram.programId, rent: SYSVAR_RENT_PUBKEY,
    } as any).rpc();
    await program.methods.createUserCard().accounts({
      vaultState, userCard: userCard(), user: admin.publicKey, systemProgram: SystemProgram.programId,
    } as any).rpc();
  });

  it("mints shares 1:1 for the first deposit", async () => {
    await mintTo(provider.connection, admin, tokenMint, userTokenAccount, admin, 1_000 * UNIT);
    await deposit(units(1_000));
    const shares = await getAccount(provider.connection, getAssociatedTokenAddressSync(shareMint, admin.publicKey));
    const vault = await program.account.vaultState.fetch(vaultState);
    assert.equal(shares.amount.toString(), String(1_000 * UNIT));
    assert.equal(vault.totalDeposited.toString(), String(1_000 * UNIT));
  });

  it("uses the share rate after the vault earns tokens", async () => {
    await mintTo(provider.connection, admin, tokenMint, userTokenAccount, admin, 210 * UNIT);
    await transfer(provider.connection, admin, userTokenAccount, vaultTokenAccount, admin, 100 * UNIT);
    await deposit(units(110));
    const shares = await getAccount(provider.connection, getAssociatedTokenAddressSync(shareMint, admin.publicKey));
    assert.equal(shares.amount.toString(), String(1_100 * UNIT));
  });

  it("rejects a zero deposit and deposits while paused", async () => {
    let zeroRejected = false;
    try { await deposit(new BN(0)); } catch { zeroRejected = true; }
    assert.isTrue(zeroRejected);
    await program.methods.setPaused(true).accounts({ vaultState, admin: admin.publicKey } as any).rpc();
    let pausedRejected = false;
    try { await deposit(units(1)); } catch { pausedRejected = true; }
    assert.isTrue(pausedRejected);
  });
});

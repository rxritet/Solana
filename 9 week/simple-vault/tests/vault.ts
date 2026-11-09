import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { createAccount, createMint, getAccount, mintTo } from "@solana/spl-token";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import { assert } from "chai";
import { SimpleVault } from "../target/types/simple_vault";

async function fund(provider: anchor.AnchorProvider, admin: Keypair, recipient: PublicKey) {
  const transaction = new Transaction().add(SystemProgram.transfer({
    fromPubkey: admin.publicKey,
    toPubkey: recipient,
    lamports: Math.floor(0.05 * LAMPORTS_PER_SOL),
  }));
  await provider.sendAndConfirm(transaction, [admin]);
}

describe("simple_vault week 10", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.SimpleVault as Program<SimpleVault>;
  const admin = (provider.wallet as anchor.Wallet).payer;
  const userA = Keypair.generate();
  const userB = Keypair.generate();
  const userWithoutCard = Keypair.generate();
  const unit = 1_000_000;

  let tokenMint: PublicKey;
  let vaultState: PublicKey;
  let vaultTokenAccount: PublicKey;
  let shareMint: PublicKey;
  const tokenAccounts: Record<string, PublicKey> = {};
  const shareAccounts: Record<string, PublicKey> = {};

  const card = (user: PublicKey) => PublicKey.findProgramAddressSync(
    [Buffer.from("user_card"), vaultState.toBuffer(), user.toBuffer()], program.programId,
  )[0];

  const deposit = (user: Keypair, amount: number) => program.methods.deposit(new anchor.BN(amount)).accounts({
    vaultState, userCard: card(user.publicKey), vaultTokenAccount, shareMint,
    userTokenAccount: tokenAccounts[user.publicKey.toBase58()],
    userShareAccount: shareAccounts[user.publicKey.toBase58()], user: user.publicKey,
  } as any).signers([user]).rpc();

  const withdraw = (user: Keypair, amount: number) => program.methods.withdraw(new anchor.BN(amount)).accounts({
    vaultState, userCard: card(user.publicKey), vaultTokenAccount, shareMint,
    userTokenAccount: tokenAccounts[user.publicKey.toBase58()],
    userShareAccount: shareAccounts[user.publicKey.toBase58()], user: user.publicKey,
  } as any).signers([user]).rpc();

  const mustFail = async (action: () => Promise<unknown>, message: string) => {
    let failed = false;
    try { await action(); } catch { failed = true; }
    assert.isTrue(failed, message);
  };

  before(async () => {
    await Promise.all([userA, userB, userWithoutCard].map((user) => fund(provider, admin, user.publicKey)));
    tokenMint = await createMint(provider.connection, admin, admin.publicKey, null, 6);
    [vaultState] = PublicKey.findProgramAddressSync([Buffer.from("vault"), tokenMint.toBuffer()], program.programId);
    [vaultTokenAccount] = PublicKey.findProgramAddressSync([Buffer.from("vault_tokens"), tokenMint.toBuffer()], program.programId);
    [shareMint] = PublicKey.findProgramAddressSync([Buffer.from("share_mint"), tokenMint.toBuffer()], program.programId);
  });

  it("initializes the vault and two user cards", async () => {
    await program.methods.initializeVault().accounts({
      vaultState, vaultTokenAccount, shareMint, tokenMint, admin: admin.publicKey,
    } as any).rpc();
    for (const user of [userA, userB]) {
      await program.methods.createUserCard().accounts({
        vaultState, userCard: card(user.publicKey), user: user.publicKey,
      } as any).signers([user]).rpc();
    }
    const vault = await program.account.vaultState.fetch(vaultState);
    assert.equal(vault.totalTokens.toNumber(), 0);
    assert.equal(vault.totalShares.toNumber(), 0);
  });

  it("deposits tokens and mints shares", async () => {
    for (const user of [userA, userB, userWithoutCard]) {
      const key = user.publicKey.toBase58();
      tokenAccounts[key] = await createAccount(provider.connection, admin, tokenMint, user.publicKey);
      shareAccounts[key] = await createAccount(provider.connection, admin, shareMint, user.publicKey);
      await mintTo(provider.connection, admin, tokenMint, tokenAccounts[key], admin, 1_000 * unit);
    }
    await deposit(userA, 100 * unit);
    await deposit(userB, 50 * unit);
    const vault = await program.account.vaultState.fetch(vaultState);
    assert.equal(vault.totalTokens.toNumber(), 150 * unit);
    assert.equal(vault.totalShares.toNumber(), 150 * unit);
    assert.equal((await getAccount(provider.connection, shareAccounts[userA.publicKey.toBase58()])).amount, BigInt(100 * unit));
    assert.equal((await getAccount(provider.connection, shareAccounts[userB.publicKey.toBase58()])).amount, BigInt(50 * unit));
    await mustFail(() => deposit(userWithoutCard, unit), "deposit without a card must fail");
  });

  it("withdraws part of a user's shares and returns tokens", async () => {
    const before = (await getAccount(provider.connection, tokenAccounts[userA.publicKey.toBase58()])).amount;
    await withdraw(userA, 50 * unit);
    const after = (await getAccount(provider.connection, tokenAccounts[userA.publicKey.toBase58()])).amount;
    assert.equal(after - before, BigInt(50 * unit));
    assert.equal((await getAccount(provider.connection, shareAccounts[userA.publicKey.toBase58()])).amount, BigInt(50 * unit));
    const vault = await program.account.vaultState.fetch(vaultState);
    assert.equal(vault.totalTokens.toNumber(), 100 * unit);
    assert.equal(vault.totalShares.toNumber(), 100 * unit);
  });

  it("withdraws all shares and rejects invalid withdrawals", async () => {
    await withdraw(userB, 50 * unit);
    assert.equal((await getAccount(provider.connection, shareAccounts[userB.publicKey.toBase58()])).amount, BigInt(0));
    assert.equal((await getAccount(provider.connection, tokenAccounts[userB.publicKey.toBase58()])).amount, BigInt(1_000 * unit));
    await mustFail(() => withdraw(userA, 0), "zero withdrawal must fail");
    await mustFail(() => withdraw(userA, 51 * unit), "withdrawal above balance must fail");
    await mustFail(() => withdraw(userWithoutCard, unit), "withdrawal without a card must fail");
  });
});

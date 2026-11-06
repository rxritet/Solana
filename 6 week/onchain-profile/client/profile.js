const fs = require("fs");
const {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} = require("@solana/web3.js");
const { Program } = require("@anchor-lang/core");

const idl = require("../target/idl/onchain_profile.json");

const PROGRAM_ID = new PublicKey(
  "WuJdhVCp71LsNuePY812xooTY4r28ogJ9ee9EnLNN8s"
);

const NAME = "Abraev";
const FIRST_BIO = "AlmaU 3-rd course student";
const UPDATED_BIO = "IT-Startup Solana Blockchain";

const connection = new Connection(
  "https://api.devnet.solana.com",
  "confirmed"
);

const secretKey = JSON.parse(
  fs.readFileSync(`${process.env.HOME}/.config/solana/id.json`, "utf8")
);
const authority = Keypair.fromSecretKey(Uint8Array.from(secretKey));

const program = new Program(idl, { connection });

async function main() {
  const [profilePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("profile"), authority.publicKey.toBuffer()],
    PROGRAM_ID
  );

  console.log("Кошелёк:", authority.publicKey.toBase58());
  console.log("PDA профиля:", profilePda.toBase58());

  try {
    const existingProfile = await program.account.userProfile.fetch(profilePda);
    console.log("Профиль уже существует:", existingProfile.name);
  } catch {
    const createInstruction = await program.methods
      .createProfile(NAME, FIRST_BIO)
      .accounts({
        authority: authority.publicKey,
        profile: profilePda,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    const createSignature = await sendAndConfirmTransaction(
      connection,
      new Transaction().add(createInstruction),
      [authority]
    );

    console.log("Профиль создан:", createSignature);
  }

  const updateInstruction = await program.methods
    .updateProfile(NAME, UPDATED_BIO)
    .accounts({
      authority: authority.publicKey,
      profile: profilePda,
    })
    .instruction();

  const updateSignature = await sendAndConfirmTransaction(
    connection,
    new Transaction().add(updateInstruction),
    [authority]
  );

  const profile = await program.account.userProfile.fetch(profilePda);

  console.log("Профиль обновлён:", updateSignature);
  console.log("Имя:", profile.name);
  console.log("Описание:", profile.bio);
  console.log("Владелец:", profile.authority.toBase58());
}

main().catch(console.error);

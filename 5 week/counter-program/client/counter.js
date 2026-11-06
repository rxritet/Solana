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

const idl = require("../target/idl/counter_program.json");

const PROGRAM_ID = new PublicKey(
  "EmnHY9cx5uVGQZnj3T3sRqHYBEaUD3ZpiSMN4D6FHcnu"
);

const connection = new Connection(
  "https://api.devnet.solana.com",
  "confirmed"
);

const secretKey = JSON.parse(
  fs.readFileSync(`${process.env.HOME}/.config/solana/id.json`, "utf8")
);
const payer = Keypair.fromSecretKey(Uint8Array.from(secretKey));

const program = new Program(idl, { connection });

async function main() {
  const [counterPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("counter")],
    PROGRAM_ID
  );

  console.log("Кошелёк:", payer.publicKey.toBase58());
  console.log("Адрес Counter PDA:", counterPda.toBase58());

  try {
    await program.account.counter.fetch(counterPda);
    console.log("Counter уже существует — инициализацию пропускаем.");
  } catch {
    const initializeInstruction = await program.methods
      .initialize()
      .accounts({
        payer: payer.publicKey,
        counter: counterPda,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    const initializeSignature = await sendAndConfirmTransaction(
      connection,
      new Transaction().add(initializeInstruction),
      [payer]
    );

    console.log("Counter создан:", initializeSignature);
  }

  const incrementInstruction = await program.methods
    .increment()
    .accounts({
      counter: counterPda,
      authority: payer.publicKey,
    })
    .instruction();

  const incrementSignature = await sendAndConfirmTransaction(
    connection,
    new Transaction().add(incrementInstruction),
    [payer]
  );

  const counter = await program.account.counter.fetch(counterPda);

  console.log("Счётчик увеличен:", incrementSignature);
  console.log("Текущее значение:", counter.count.toString());
}

main().catch(console.error);

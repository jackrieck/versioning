import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Versioning, IDL } from "../target/types/versioning";

describe("versioning", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = new anchor.Program(
    IDL,
    new anchor.web3.PublicKey("8EEY7nX8xNTgNHvsYAhKZ8TwLP6eJLEhBZGbJWx3vtD6"),
    anchor.getProvider()
  );

  it.skip("Init V1", async () => {
    const [data, _dataBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("data")],
      program.programId
    );

    const initializeArgs = {
      foo: new anchor.BN(1),
    };

    const initializeTxSig = await program.methods
      .initialize(initializeArgs)
      .accounts({
        data: data,
        payer: program.provider.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("initializeTxSig: %s", initializeTxSig);

    const dataAccount = await program.account.data.fetch(data, "processed");
    console.log(JSON.stringify(dataAccount)); 
  });

  it("Migrate V1 to V2", async () => {
    const [data, _dataBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("data")],
      program.programId
    );

    const dataAccountPre = await program.account.data.fetch(data, "processed");
    console.log(JSON.stringify(dataAccountPre));

    const migrateTxSig = await program.methods
      .migrate()
      .accounts({
        data: data,
        payer: program.provider.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc({ skipPreflight: false });
    console.log("migrateTxSig: %s", migrateTxSig);

    const dataAccountPost = await program.account.data.fetch(data, "processed");
    console.log(JSON.stringify(dataAccountPost));
  });
});

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { GetCommitmentSignature } from "@magicblock-labs/ephemeral-rollups-sdk";
import { ErStateAccount } from "../target/types/er_state_account";
import {
  init,
  taskKey,
  taskQueueAuthorityKey,
} from "@helium/tuktuk-sdk";
import { assert } from "chai";

describe("er-state-account", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const providerEphemeralRollup = new anchor.AnchorProvider(
    new anchor.web3.Connection(
      process.env.EPHEMERAL_PROVIDER_ENDPOINT ||
      "https://devnet.magicblock.app/",
      {
        wsEndpoint:
          process.env.EPHEMERAL_WS_ENDPOINT || "wss://devnet.magicblock.app/",
      },
    ),
    anchor.Wallet.local(),
  );
  console.log("Base Layer Connection: ", provider.connection.rpcEndpoint);
  console.log(
    "Ephemeral Rollup Connection: ",
    providerEphemeralRollup.connection.rpcEndpoint,
  );
  console.log(`Current SOL Public Key: ${anchor.Wallet.local().publicKey}`);



  before(async function () {
    const balance = await provider.connection.getBalance(
      anchor.Wallet.local().publicKey,
    );
    console.log("Current balance is", balance / LAMPORTS_PER_SOL, " SOL", "\n");
  });

  const program = anchor.workspace.erStateAccount as Program<ErStateAccount>;
  const ephemeralProgram = new Program(program.idl, providerEphemeralRollup);
  const userAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user"), anchor.Wallet.local().publicKey.toBuffer()],
    program.programId,
  )[0];

  const taskQueue = new anchor.web3.PublicKey("9kkhV4szyGJdZRV5joZEeJJN6Qsyo8X5827KBmnCMzKE");
  const queueAuthority = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("queue_authority")], program.programId)[0];
  const taskQueueAuthority = taskQueueAuthorityKey(taskQueue, queueAuthority)[0];

  xit("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods
      .initialize()
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("User Account initialized: ", tx);
  });

  xit("Update State!", async () => {
    const tx = await program.methods
      .update(new anchor.BN(42))
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
      })
      .rpc();
    console.log("\nUser Account State Updated: ", tx);
  });

  xit("Schedule increment task", async () => {
    let tuktukProgram = await init(provider);
    console.log(userAccount)
    console.log(queueAuthority)
    console.log(taskQueue)
    console.log(taskQueueAuthority)
    let taskID = 2;
    const tx = await program.methods.schedule(taskID)
      .accountsPartial({
        user: provider.publicKey,
        userAccount: userAccount,
        oracleQueue: new PublicKey("Cuj97ggrhhidhbu39TijNVqE74xvKJ69gDervRUXAxGh"),
        taskQueue: taskQueue,
        taskQueueAuthority: taskQueueAuthority,
        task: taskKey(taskQueue, taskID)[0],
        queueAuthority: queueAuthority,
        systemProgram: anchor.web3.SystemProgram.programId,
        tuktukProgram: tuktukProgram.programId,
      })
      .rpc();
    assert.isTrue(tuktukProgram.programId.equals(new anchor.web3.PublicKey("tuktukUrfhXT6ZT77QTU8RQtvgL967uRuVagWF57zVA")));
    console.log("\nYour transaction signature", tx);

    console.log("Waiting for VRF callback...");
    await new Promise((resolve) => setTimeout(resolve, 10000));

    const account = await program.account.userAccount.fetch(userAccount);
    console.log("Random value: ", account.data.toString());;
  });

  it("print random value", async () => {


    const account = await program.account.userAccount.fetch(userAccount);
    console.log("Random value: ", account.data.toString());;
  });

  xit("Call VRF on base layer", async () => {
    await program.methods.generateRandomData(1)
      .accountsPartial({
        oracleQueue: new PublicKey("Cuj97ggrhhidhbu39TijNVqE74xvKJ69gDervRUXAxGh")
      }).rpc();

    console.log("Waiting for VRF callback...");
    await new Promise((resolve) => setTimeout(resolve, 10000));

    const account = await program.account.userAccount.fetch(userAccount);
    console.log("Random value: ", account.data.toString());
  });

  xit("Delegate to Ephemeral Rollup!", async () => {
    let tx = await program.methods
      .delegate()
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
        validator: new PublicKey("MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57"),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc({ skipPreflight: true });

    console.log("\nUser Account Delegated to Ephemeral Rollup: ", tx);
  });

  xit("Execute VRF Delegated", async () => {
    let tx = await ephemeralProgram.methods.generateRandomData(0).accountsPartial({
      oracleQueue: new PublicKey("5hBR571xnXppuCPveTrctfTU7tJLSN94nq7kv7FRK5Tc")
    }).transaction();
    tx.feePayer = providerEphemeralRollup.wallet.publicKey;

    tx.recentBlockhash = (
      await providerEphemeralRollup.connection.getLatestBlockhash()
    ).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);
    const txHash = await providerEphemeralRollup.sendAndConfirm(tx, [], {
      skipPreflight: false,
    });
    console.log("Your transaction signature", tx);



    console.log("Waiting for VRF callback...");
    await new Promise((resolve) => setTimeout(resolve, 10000));
    let account = await providerEphemeralRollup.connection.getAccountInfo(
      userAccount,
    );
    // console.log("User Account Info: ", account);

    // const randomValue = new anchor.BN(account.data.slice(40, 48), "le");
    // console.log("Random value: ", randomValue.toString());

  });

  xit("Update State and Commit to Base Layer!", async () => {
    let tx = await program.methods
      .updateCommit(new anchor.BN(43))
      .accountsPartial({
        user: providerEphemeralRollup.wallet.publicKey,
        userAccount: userAccount,
      })
      .transaction();

    tx.feePayer = providerEphemeralRollup.wallet.publicKey;

    tx.recentBlockhash = (
      await providerEphemeralRollup.connection.getLatestBlockhash()
    ).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);
    const txHash = await providerEphemeralRollup.sendAndConfirm(tx, [], {
      skipPreflight: false,
    });
    const txCommitSgn = await GetCommitmentSignature(
      txHash,
      providerEphemeralRollup.connection,
    );

    console.log("\nUser Account State Updated: ", txHash);
  });

  xit("Commit and undelegate from Ephemeral Rollup!", async () => {
    let info = await providerEphemeralRollup.connection.getAccountInfo(
      userAccount,
    );

    console.log("User Account Info: ", info);

    console.log("User account", userAccount.toBase58());

    let tx = await program.methods
      .undelegate()
      .accounts({
        user: providerEphemeralRollup.wallet.publicKey,
      })
      .transaction();

    tx.feePayer = providerEphemeralRollup.wallet.publicKey;

    tx.recentBlockhash = (
      await providerEphemeralRollup.connection.getLatestBlockhash()
    ).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);
    const txHash = await providerEphemeralRollup.sendAndConfirm(tx, [], {
      skipPreflight: false,
    });
    const txCommitSgn = await GetCommitmentSignature(
      txHash,
      providerEphemeralRollup.connection,
    );

    console.log("\nUser Account Undelegated: ", txHash);
  });

  xit("Update State!", async () => {
    let tx = await program.methods
      .update(new anchor.BN(45))
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
      })
      .rpc();

    console.log("\nUser Account State Updated: ", tx);
  });

  xit("Close Account!", async () => {
    const tx = await program.methods
      .close()
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("\nUser Account Closed: ", tx);
  });
});

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  init,
  taskKey,
  taskQueueAuthorityKey,
} from "@helium/tuktuk-sdk";
import { GptOracleTuktuk } from "../target/types/gpt_oracle_tuktuk";
import { SolanaGptOracle } from "../solana_gpt_oracle";
import IDL_LLM from "../solana_gpt_oracle.json";
import { assert } from "chai";

describe("gpt-oracle-tuktuk", () => {
  // Configure the client to use the local cluster.

  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);
  const program = anchor.workspace.gptOracleTuktuk as Program<GptOracleTuktuk>;
  const program_llm = new anchor.Program(
    IDL_LLM as anchor.Idl,
    provider
  ) as Program<SolanaGptOracle>;

  const taskQueue = new anchor.web3.PublicKey("DzAr6B5xeAy9zTDhpC4DkiLB5JpCsFYkC7F1dtPe6LCf");
  const queueAuthority = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("queue_authority")], program.programId)[0];
  const taskQueueAuthority = taskQueueAuthorityKey(taskQueue, queueAuthority)[0];

  async function GetAgentAndInteraction(
    program: Program<GptOracleTuktuk>,
    provider: anchor.AnchorProvider,
    program_llm: Program<SolanaGptOracle>
  ) {
    const agentAddress = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("agent")],
      program.programId
    )[0];

    const agent = await program.account.agent.fetch(agentAddress);

    // Interaction Address
    const interactionAddress = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("interaction"),
        provider.wallet.publicKey.toBuffer(),
        agent.context.toBuffer(),
      ],
      program_llm.programId
    )[0];
    return { agent, interactionAddress, agentAddress };
  }
  xit("Initialize Agent", async () => {
    const counterAddress = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("counter")],
      program_llm.programId
    )[0];
    const counter = await program_llm.account.counter.fetch(counterAddress);
    const contextAddress = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("test-context"),
        new anchor.BN(counter.count).toArrayLike(Buffer, "le", 4),
      ],
      program_llm.programId
    )[0];
    const agentAddress = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("agent")],
      program.programId
    )[0];

    const tx = await program.methods
      .initialize()
      .accountsPartial({
        payer: provider.wallet.publicKey,
        agent: agentAddress,
        llmContext: contextAddress,
        counter: counterAddress,
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });

  it("Schedule agent interaction", async () => {
    let tuktukProgram = await init(provider);

    console.log(queueAuthority)
    console.log(taskQueue)
    console.log(taskQueueAuthority)
    const { agent, interactionAddress, agentAddress } = await GetAgentAndInteraction(
      program,
      provider,
      program_llm
    );

    console.log(agent)
    console.log(interactionAddress)
    console.log(agentAddress)
    const fundTx = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: queueAuthority,
        lamports: 28953600,
      })
    );
    await provider.sendAndConfirm(fundTx);
    let taskID = 4;
    const prompt = "Give me word of the day"
    const tx = await program.methods.schedule(prompt, taskID)
      .accountsPartial({
        user: provider.wallet.publicKey,
        interaction: interactionAddress,
        agent: agentAddress,
        contextAccount: agent.context,
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

  });


  xit("close agent", async () => {
    const agentAddress = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("agent")],
      program.programId
    )[0];
    const tx = await program.methods
      .closeAgent()
      .accountsPartial({
        payer: provider.wallet.publicKey,
        agent: agentAddress
      })
      .rpc();

    console.log("Your transaction signature", tx);
  });
});

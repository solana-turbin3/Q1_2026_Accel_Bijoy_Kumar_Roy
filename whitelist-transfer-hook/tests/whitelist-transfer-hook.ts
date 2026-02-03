import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createInitializeMintInstruction,
  getMintLen,
  ExtensionType,
  createTransferCheckedWithTransferHookInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createInitializeTransferHookInstruction,
  createAssociatedTokenAccountInstruction,
  createMintToInstruction,
  createTransferCheckedInstruction,
} from "@solana/spl-token";
import {
  LAMPORTS_PER_SOL,
  SendTransactionError,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction
} from '@solana/web3.js';
import { WhitelistTransferHook } from "../target/types/whitelist_transfer_hook";

describe("whitelist-transfer-hook", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = anchor.web3.Keypair.generate();
  const anotherWallet = anchor.web3.Keypair.generate()

  const program = anchor.workspace.whitelistTransferHook as Program<WhitelistTransferHook>;

  const mint2022 = anchor.web3.Keypair.generate();

  // Sender token account address
  const sourceTokenAccount = getAssociatedTokenAddressSync(
    mint2022.publicKey,
    wallet.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  // Recipient token account address
  const recipient = anchor.web3.Keypair.generate();
  const destinationTokenAccount = getAssociatedTokenAddressSync(
    mint2022.publicKey,
    recipient.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  // ExtraAccountMetaList address
  // Store extra accounts required by the custom transfer hook instruction
  const [extraAccountMetaListPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('extra-account-metas'), mint2022.publicKey.toBuffer()],
    program.programId,
  );
  function createWhiteListPda(wallet: anchor.web3.PublicKey) {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("whitelist"),
        wallet.toBuffer()
      ],
      program.programId
    )[0];

  }

  before(async () => {
    const sig = await provider.connection.requestAirdrop(wallet.publicKey, 100 * LAMPORTS_PER_SOL)
    await provider.connection.confirmTransaction(sig, "confirmed");
  })


  it("Add user to whitelist", async () => {
    const whiteListPda1 = createWhiteListPda(wallet.publicKey)
    const whiteListPda2 = createWhiteListPda(anotherWallet.publicKey)
    const tx = await program.methods.initializeWhitelist(wallet.publicKey)
      .accountsPartial({
        admin: wallet.publicKey,
        whitelist: whiteListPda1,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).signers([wallet])
      .rpc();

    await program.methods.initializeWhitelist(anotherWallet.publicKey)
      .accountsPartial({
        admin: wallet.publicKey,
        whitelist: whiteListPda2,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).signers([wallet])
      .rpc();

    const whiteListPda1Account = await program.account.whitelist.fetch(whiteListPda1);
    console.log(whiteListPda1Account)

    console.log("\nUser added to whitelist:", wallet.publicKey.toBase58());
    console.log("\nUser added to whitelist:", provider.publicKey.toBase58());
    console.log("Transaction signature:", tx);
  });

  it("Remove user to whitelist", async () => {
    const whitelist = createWhiteListPda(wallet.publicKey)
    const tx = await program.methods.removeFromWhitelist()
      .accountsPartial({
        admin: wallet.publicKey,
        user: wallet.publicKey,
        whitelist,
      }).signers([wallet])
      .rpc();

    console.log("\nUser removed from whitelist:", provider.publicKey.toBase58());
    console.log("Transaction signature:", tx);
  });


  it("init and mint", async () => {

    const amount = new anchor.BN(2 * LAMPORTS_PER_SOL);

    await program.methods
      .initAndMint(amount)
      .accountsPartial({
        user: wallet.publicKey,
        mint: mint2022.publicKey,
        userTokenAccount: sourceTokenAccount,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
      })
      .signers([wallet, mint2022])
      .rpc();
    const accountInfo = await provider.connection.getAccountInfo(mint2022.publicKey);


    if (!accountInfo) {
      throw new Error("Mint account not found");
    }

    if (!accountInfo.owner.equals(TOKEN_2022_PROGRAM_ID)) {
      throw new Error("Mint is NOT owned by Token-2022 program");
    }
  })
  it('Create Token Accounts', async () => {


    const transaction = new Transaction().add(

      createAssociatedTokenAccountInstruction(
        wallet.publicKey,
        destinationTokenAccount,
        recipient.publicKey,
        mint2022.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
      ),

    );

    const txSig = await sendAndConfirmTransaction(provider.connection, transaction, [wallet], { skipPreflight: true });

    console.log("\nTransaction Signature: ", txSig);
  });



  // Account to store extra accounts required by the transfer hook instruction
  it('Create ExtraAccountMetaList Account', async () => {
    const initializeExtraAccountMetaListInstruction = await program.methods
      .initializeTransferHook()
      .accountsPartial({
        payer: wallet.publicKey,
        mint: mint2022.publicKey,
        extraAccountMetaList: extraAccountMetaListPDA,
        systemProgram: SystemProgram.programId,
      }).signers([wallet])
      //.instruction();
      .rpc();

    //const transaction = new Transaction().add(initializeExtraAccountMetaListInstruction);

    //const txSig = await sendAndConfirmTransaction(provider.connection, transaction, [wallet.payer], { skipPreflight: true, commitment: 'confirmed' });
    console.log("\nExtraAccountMetaList Account created:", extraAccountMetaListPDA.toBase58());
    console.log('Transaction Signature:', initializeExtraAccountMetaListInstruction);
  });

  it('Transfer Hook with Extra Account Meta', async () => {
    // 1 tokens
    const amount = 1 * 10 ** 9;
    const amountBigInt = BigInt(amount);


    const transferInstruction = await createTransferCheckedWithTransferHookInstruction(
      provider.connection,
      sourceTokenAccount,
      mint2022.publicKey,
      destinationTokenAccount,
      wallet.publicKey,
      amountBigInt,
      9,
      [],
      "confirmed",
      TOKEN_2022_PROGRAM_ID
    );
    const transaction = new Transaction().add(transferInstruction);

    try {
      // Send the transaction
      const txSig = await sendAndConfirmTransaction(provider.connection, transaction, [wallet], { skipPreflight: false });
      console.log("\nTransfer Signature:", txSig);
    }
    catch (error) {
      if (error instanceof SendTransactionError) {
        console.error("\nTransaction failed:", error.logs[6]);
        // console.error("\nTransaction failed. Full logs:");
        // error.logs?.forEach((log, i) => console.error(`  ${i}: ${log}`));
      } else {
        console.error("\nUnexpected error:", error);
      }
    }
  });
});

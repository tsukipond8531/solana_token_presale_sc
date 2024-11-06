import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import * as assert from "assert";
import { SolanaTokenPresale} from "../target/types/solana_token_presale";

// Define an interface for the Presale account
interface PresaleAccount {
  totalDeposits: anchor.BN;
  admin: anchor.web3.PublicKey;
}

interface UserBalanceAccount {
  amount: anchor.BN;
}

describe("solana_token_presale", () => {
  const provider = AnchorProvider.env();
  anchor.setProvider(provider);

  // Initialize the program with an explicit type cast
  const program = anchor.workspace.SolanaTokenPresale as Program<SolanaTokenPresale>
  
  let presaleAccount: anchor.web3.Keypair;
  let userAccount: anchor.web3.Keypair;
  let userBalanceAccount: anchor.web3.Keypair;

  before(async () => {
    presaleAccount = anchor.web3.Keypair.generate();
    userAccount = anchor.web3.Keypair.generate();
    userBalanceAccount = anchor.web3.Keypair.generate();

    // Fund the user account with some SOL for testing
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(userAccount.publicKey, anchor.web3.LAMPORTS_PER_SOL * 2)
    );
  });

  it("Initializes the presale account", async () => {
    await program.methods.initialize()
      .accounts({
        presale: presaleAccount.publicKey,
        admin: provider.wallet.publicKey,
      })
      .signers([presaleAccount])
      .rpc();

    // Fetch the presale account and verify values
    const presale = await program.account.presale.fetch(presaleAccount.publicKey) as PresaleAccount;
    assert.equal(presale.totalDeposits.toNumber(), 0);
    assert.ok(presale.admin.equals(provider.wallet.publicKey));
  });

  it("Deposits lamports into the presale account", async () => {
    const depositAmount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL);

    await program.methods.deposit(depositAmount)
      .accounts({
        presale: presaleAccount.publicKey,
        user: userAccount.publicKey,
      })
      .signers([userAccount])
      .rpc();

    // Fetch and validate the presale and user balance accounts
    const presale = await program.account.presale.fetch(presaleAccount.publicKey) as PresaleAccount;
    const userBalance = await program.account.userBalance.fetch(userBalanceAccount.publicKey) as UserBalanceAccount;

    assert.equal(userBalance.amount.toNumber(), depositAmount.toNumber());
    assert.equal(presale.totalDeposits.toNumber(), depositAmount.toNumber());
  });

  it("Checks the balance of a user", async () => {
    const userBalance = await program.account.userBalance.fetch(userBalanceAccount.publicKey) as UserBalanceAccount;
    console.log("User balance:", userBalance.amount.toNumber());
    assert.ok(userBalance.amount.toNumber() > 0);
  });

  it("Allows the admin to withdraw funds", async () => {
    const presaleBefore = await provider.connection.getBalance(presaleAccount.publicKey);
    const adminBefore = await provider.connection.getBalance(provider.wallet.publicKey);

    await program.methods.withdraw()
      .accounts({
        presale: presaleAccount.publicKey,
        admin: provider.wallet.publicKey,
      })
      .rpc();

    const presaleAfter = await provider.connection.getBalance(presaleAccount.publicKey);
    const adminAfter = await provider.connection.getBalance(provider.wallet.publicKey);

    assert.equal(presaleAfter, 0);
    assert.ok(adminAfter > adminBefore);
  });
});

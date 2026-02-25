import {
  Horizon,
  TransactionBuilder,
  BASE_FEE,
  Operation,
  Asset,
  Keypair,
} from "@stellar/stellar-sdk";
import { env } from "../config/env";

export class BlockchainService {
  private server: Horizon.Server;
  private networkPassphrase: string;

  constructor() {
    this.server = new Horizon.Server(env.stellar.horizonUrl);
    this.networkPassphrase = env.stellar.networkPassphrase;
  }

  /**
   * Returns the balance string for a given asset on the account.
   * Pass `"native"` or `"XLM"` for the native lumen balance.
   */
  async getAccountBalance(
    address: string,
    assetCode: string = "native",
  ): Promise<string> {
    const account = await this.server.loadAccount(address);

    if (assetCode === "native" || assetCode === "XLM") {
      const native = account.balances.find(
        (b: Horizon.HorizonApi.BalanceLine) => b.asset_type === "native",
      );
      return native?.balance ?? "0";
    }

    const token = account.balances.find(
      (b: Horizon.HorizonApi.BalanceLine) =>
        "asset_code" in b && b.asset_code === assetCode,
    );
    return token?.balance ?? "0";
  }

  /**
   * Builds an XDR for a native XLM payment, fee-sponsored by the backend.
   * Primarily used for funding minimum reserves on new accounts or
   * trustline creation.
   *
   * The Fee Payer is the transaction source (pays fees); the `from` address
   * is set as the operation source. If `from` equals the Fee Payer the
   * returned XDR is fully signed; otherwise the caller must also collect
   * the `from` account's signature.
   */
  async buildNativePayment(
    from: string,
    to: string,
    amount: string,
  ): Promise<string> {
    const feePayerAccount = await this.server.loadAccount(
      env.feePayer.publicKey,
    );

    const tx = new TransactionBuilder(feePayerAccount, {
      fee: BASE_FEE,
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(
        Operation.payment({
          destination: to,
          asset: Asset.native(),
          amount,
          source: from,
        }),
      )
      .setTimeout(30)
      .build();

    const feePayerKeypair = Keypair.fromSecret(env.feePayer.secretKey);
    tx.sign(feePayerKeypair);

    return tx.toXDR();
  }

  /**
   * Builds an XDR for a token payment sponsoring the transaction fee.
   * Part of the Account Abstraction (AA) flow — the Fee Payer is the
   * outer transaction source while the user signs for the operation.
   */
  async buildSponsoredPaymentXDR(
    from: string,
    to: string,
    amount: string,
    assetCode: string = "USDC",
    assetIssuer?: string,
  ): Promise<string> {
    const feePayerAccount = await this.server.loadAccount(
      env.feePayer.publicKey,
    );

    if (assetCode !== "native" && assetCode !== "XLM" && !assetIssuer) {
      throw new Error(
        `assetIssuer is required for non-native asset "${assetCode}"`,
      );
    }

    const asset =
      assetCode === "native" || assetCode === "XLM"
        ? Asset.native()
        : new Asset(assetCode, assetIssuer!);

    const tx = new TransactionBuilder(feePayerAccount, {
      fee: BASE_FEE,
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(
        Operation.payment({
          destination: to,
          asset,
          amount,
          source: from,
        }),
      )
      .setTimeout(30)
      .build();

    const feePayerKeypair = Keypair.fromSecret(env.feePayer.secretKey);
    tx.sign(feePayerKeypair);

    return tx.toXDR();
  }

  /**
   * Fetch transaction history for an account from Horizon (for risk scoring).
   * Returns up to 200 most recent transactions.
   */
  async getTransactionHistory(
    accountId: string,
    limit: number = 200,
  ): Promise<Horizon.HorizonApi.TransactionResponse[]> {
    const coll = (this.server as any).transactions();
    const builder = coll.forAccount(accountId).order("desc").limit(limit);
    const resp = await builder.call();
    return resp.records ?? [];
  }
}

export default new BlockchainService();

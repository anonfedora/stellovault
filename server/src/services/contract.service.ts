import {
  Contract,
  xdr,
  nativeToScVal,
  scValToNative,
  TransactionBuilder,
  BASE_FEE,
  Keypair,
  Operation,
  Horizon,
  SorobanRpc,
  Transaction,
} from "@stellar/stellar-sdk";
import { env } from "../config/env";
import { contracts } from "../config/contracts";

const MAX_POLL_ATTEMPTS = 30;
const POLL_INTERVAL_MS = 1000;

export class ContractService {
  private horizonServer: Horizon.Server;
  private rpcServer: SorobanRpc.Server;
  private networkPassphrase: string;

  constructor() {
    this.horizonServer = new Horizon.Server(env.stellar.horizonUrl);
    this.rpcServer = new SorobanRpc.Server(env.stellar.rpcUrl);
    this.networkPassphrase = env.stellar.networkPassphrase;
  }

  /**
   * Builds a Soroban contract invocation XDR using the Account Abstraction
   * pattern: the Fee Payer is the outer transaction source (pays fees and
   * provides the sequence number) while the user's key is the operation
   * source. The returned XDR is already signed by the Fee Payer — the
   * client only needs to sign the authorization entries with their key.
   */
  async buildContractInvokeXDR(
    contractId: string,
    method: string,
    args: xdr.ScVal[],
    sourcePublicKey: string,
  ): Promise<string> {
    const contract = new Contract(contractId);
    const feePayerAccount = await this.horizonServer.loadAccount(
      env.feePayer.publicKey,
    );

    const tx = new TransactionBuilder(feePayerAccount, {
      fee: BASE_FEE,
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(
        Operation.invokeHostFunction({
          func: xdr.HostFunction.hostFunctionTypeInvokeContract(
            new xdr.InvokeContractArgs({
              contractAddress: contract.address().toScAddress(),
              functionName: method,
              args,
            }),
          ),
          auth: [],
          source: sourcePublicKey,
        }),
      )
      .setTimeout(300)
      .build();

    const simulated = await this.rpcServer.simulateTransaction(tx);

    if (SorobanRpc.Api.isSimulationRestore(simulated)) {
      throw new Error("Contract state needs restoration before invocation");
    }
    if (SorobanRpc.Api.isSimulationError(simulated)) {
      throw new Error(`Simulation failed: ${simulated.error}`);
    }

    const assembled = SorobanRpc.assembleTransaction(tx, simulated).build();

    const feePayerKeypair = Keypair.fromSecret(env.feePayer.secretKey);
    (tx as Transaction).sign(feePayerKeypair);
    // const feePayerKeypair = Keypair.fromSecret(env.feePayer.secretKey);
    // assembled.sign(feePayerKeypair);

    return assembled.toXDR();
  }

  /**
   * Simulates a read-only contract call via Soroban RPC and decodes the
   * return value with `scValToNative`.
   */
  async simulateCall(
    contractId: string,
    method: string,
    args: xdr.ScVal[],
  ): Promise<unknown> {
    const contract = new Contract(contractId);
    const sourceAccount = await this.horizonServer.loadAccount(
      env.feePayer.publicKey,
    );

    const tx = new TransactionBuilder(sourceAccount, {
      fee: BASE_FEE,
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(contract.call(method, ...args))
      .setTimeout(30)
      .build();

    const simulated = await this.rpcServer.simulateTransaction(tx);

    if (SorobanRpc.Api.isSimulationError(simulated)) {
      throw new Error(`Simulation failed: ${simulated.error}`);
    }

    if (!SorobanRpc.Api.isSimulationSuccess(simulated)) {
      throw new Error("Simulation did not return a successful result");
    }

    const returnValue = simulated.result?.retval;
    if (!returnValue) {
      return null;
    }

    return scValToNative(returnValue);
  }

  /**
   * Submits a fully-signed XDR envelope to the Soroban RPC and polls
   * until the transaction reaches a terminal state (SUCCESS / FAILED).
   */
  async submitXDR(
    signedXDR: string,
  ): Promise<{ hash: string; status: string }> {
    const tx = TransactionBuilder.fromXDR(signedXDR, this.networkPassphrase);
    const sendResponse = await this.rpcServer.sendTransaction(tx);

    if (sendResponse.status === "ERROR") {
      throw new Error(
        `Transaction rejected by network: ${sendResponse.errorResult?.toXDR("base64") ?? "unknown error"}`,
      );
    }

    let attempts = 0;
    let getResponse = await this.rpcServer.getTransaction(sendResponse.hash);

    while (
      getResponse.status === SorobanRpc.Api.GetTransactionStatus.NOT_FOUND
    ) {
      if (++attempts >= MAX_POLL_ATTEMPTS) {
        throw new Error(
          `Transaction confirmation timed out after ${MAX_POLL_ATTEMPTS}s`,
        );
      }
      await new Promise((r) => setTimeout(r, POLL_INTERVAL_MS));
      getResponse = await this.rpcServer.getTransaction(sendResponse.hash);
    }

    if (getResponse.status === SorobanRpc.Api.GetTransactionStatus.FAILED) {
      throw new Error(
        `Transaction failed on-chain: ${getResponse.resultXdr?.toXDR("base64") ?? "no result XDR available"}`,
      );
    }

    return {
      hash: sendResponse.hash,
      status: getResponse.status,
    };
  }

  /**
   * Submits a signed transaction and returns the transaction hash.
   * Used by the transaction queue service.
   */
  async submitSignedTransaction(signedXDR: string): Promise<string> {
    const result = await this.submitXDR(signedXDR);
    return result.hash;
  }

  async getEscrowState(
    escrowId: string | number,
  ): Promise<Record<string, unknown> | null> {
    const contractId = contracts.escrow?.trim();
    if (!contractId) {
      throw new Error("ESCROW_CONTRACT_ID not configured");
    }

    const numericEscrowId = Number(escrowId);
    if (!Number.isInteger(numericEscrowId) || numericEscrowId < 0) {
      throw new Error(
        "Escrow on-chain identifier must be a non-negative integer",
      );
    }

    return (await this.simulateCall(contractId, "get_escrow", [
      nativeToScVal(BigInt(numericEscrowId), { type: "u64" }),
    ])) as Record<string, unknown> | null;
  }
}

export default new ContractService();

import { Networks, Horizon } from "@stellar/stellar-sdk";

export class BlockchainService {
    private server: Horizon.Server;
    private network: string;

    constructor() {
        this.server = new Horizon.Server(process.env.HORIZON_URL || "https://horizon-testnet.stellar.org");
        this.network = process.env.STELLAR_NETWORK === "testnet" ? Networks.TESTNET : Networks.PUBLIC;
    }

    /**
     * Builds an XDR for a simple payment sponsoring the transaction fee.
     * This is part of the Account Abstraction (AA) flow.
     */
    async buildSponsoredPaymentXDR(from: string, to: string, amount: string, assetCode: string = "USDC") {
        // Logic to build XDR with backend as Fee Payer
        // Returns unsigned XDR for client-side signing
        console.log(`Building XDR for ${amount} ${assetCode} from ${from} to ${to}`);
        // Placeholder for actual SDK implementation
        return "BASE64_XDR_PLACEHOLDER";
    }

    async getAccountBalance(address: string, assetCode: string) {
        const account = await this.server.loadAccount(address);
        return account.balances.find((b: any) => b.asset_code === assetCode);
    }

    /**
     * Fetch transaction history for an account from Horizon (for risk scoring).
     * Returns up to 200 most recent transactions.
     */
    async getTransactionHistory(accountId: string, limit: number = 200): Promise<Horizon.TransactionResponse[]> {
        const coll = (this.server as any).transactions();
        const builder = coll.forAccount(accountId).order("desc").limit(limit);
        const resp = await builder.call();
        return resp.records ?? [];
    }
}

export default new BlockchainService();

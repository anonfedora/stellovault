import { env } from "./env";

/**
 * All deployed Soroban contract IDs.
 * Set via environment variables to support testnet vs mainnet.
 */
export const contracts = {
    loan: process.env.LOAN_CONTRACT_ID || "",
    collateral: process.env.COLLATERAL_CONTRACT_ID || "",
    escrow: process.env.ESCROW_CONTRACT_ID || "",
    registry: process.env.REGISTRY_CONTRACT_ID || "",
    governance: process.env.GOVERNANCE_CONTRACT_ID || "",
};

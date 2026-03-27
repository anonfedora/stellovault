import dotenv from "dotenv";
dotenv.config();

export const env = {
    port: parseInt(process.env.PORT || "3001", 10),
    databaseUrl: process.env.DATABASE_URL || "",

    stellar: {
        network: process.env.STELLAR_NETWORK || "testnet",
        horizonUrl: process.env.HORIZON_URL || "https://horizon-testnet.stellar.org",
        rpcUrl: process.env.RPC_URL || "https://soroban-testnet.stellar.org",
        networkPassphrase:
            process.env.NETWORK_PASSPHRASE || "Test SDF Network ; September 2015",
    },

    feePayer: {
        publicKey: process.env.FEE_PAYER_PUBLIC || "",
        secretKey: process.env.FEE_PAYER_SECRET || "",
        minBalance: parseFloat(process.env.FEE_PAYER_MIN_BALANCE || "50"),
    },

    treasury: {
        secretKey: process.env.TREASURY_SECRET || "",
    },

    vault: {
        address: process.env.VAULT_ADDR || "http://127.0.0.1:8200",
        token: process.env.VAULT_TOKEN || "",
    },

    jwt: {
        accessSecret: process.env.JWT_ACCESS_SECRET || "change-me-in-prod",
        refreshSecret: process.env.JWT_REFRESH_SECRET || "change-me-in-prod-refresh",
        accessExpiresIn: process.env.JWT_ACCESS_EXPIRES_IN || "15m",
        refreshExpiresIn: process.env.JWT_REFRESH_EXPIRES_IN || "7d",
    },

    webhookSecret: process.env.WEBHOOK_SECRET || "",
    corsAllowedOrigins: process.env.CORS_ALLOWED_ORIGINS?.split(",") || ["*"],
};

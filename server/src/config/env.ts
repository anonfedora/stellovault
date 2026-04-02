import dotenv from "dotenv";
dotenv.config();

export const env = {
  port: parseInt(process.env.PORT || "3001", 10),
  databaseUrl: process.env.DATABASE_URL || "",
  appBaseUrl: process.env.APP_BASE_URL || "https://stellovault.com",

  redis: {
    host: process.env.REDIS_HOST || "localhost",
    port: parseInt(process.env.REDIS_PORT || "6379", 10),
  },

  stellar: {
    network: process.env.STELLAR_NETWORK || "testnet",
    horizonUrl:
      process.env.HORIZON_URL || "https://horizon-testnet.stellar.org",
    rpcUrl: process.env.RPC_URL || "https://soroban-testnet.stellar.org",
    networkPassphrase:
      process.env.NETWORK_PASSPHRASE || "Test SDF Network ; September 2015",
  },

  feePayer: {
    publicKey: process.env.FEE_PAYER_PUBLIC || "",
    secretKey: process.env.FEE_PAYER_SECRET || "",
  },

  jwt: {
    accessSecret: process.env.JWT_ACCESS_SECRET || "change-me-in-prod",
    refreshSecret:
      process.env.JWT_REFRESH_SECRET || "change-me-in-prod-refresh",
    accessExpiresIn: process.env.JWT_ACCESS_EXPIRES_IN || "15m",
    refreshExpiresIn: process.env.JWT_REFRESH_EXPIRES_IN || "7d",
  },

  webhookSecret: process.env.WEBHOOK_SECRET || "",
  webhookSignatureSecret:
    process.env.WEBHOOK_SIGNATURE_SECRET || process.env.WEBHOOK_SECRET || "",
  corsAllowedOrigins: process.env.CORS_ALLOWED_ORIGINS?.split(",") || ["*"],
};

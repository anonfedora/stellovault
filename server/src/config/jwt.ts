/**
 * JWT configuration constants and utilities.
 */

export const JWT_CONFIG = {
    // Access token: short-lived (15 minutes by default)
    ACCESS_TOKEN_EXPIRY: "15m",
    // Refresh token: long-lived (7 days by default)
    REFRESH_TOKEN_EXPIRY: "7d",
    // Challenge nonce: valid for 5 minutes
    CHALLENGE_EXPIRY_SECONDS: 300,
};

export type TokenType = "access" | "refresh";

export interface TokenPayload {
    userId: string;
    jti: string;
    walletAddress: string;
    iat?: number;
    exp?: number;
}

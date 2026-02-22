# Features: Build Login / Connect Wallet Page

## Description
This PR implements the core authentication flow for StelloVault, allowing users to connect their Stellar wallet (Freighter) and authenticate via a challenge-response mechanism.

### Key Changes
- **Dependencies**: Added `@stellar/freighter-api`, `@stellar/stellar-sdk`, and `jose`.
- **Auth Utils**: Created `lib/auth.ts` for JWT handling and `hooks/useWalletAuth.ts` for managing the auth state.
- **API Routes**:
  - `POST /api/v1/auth/challenge`: Generates a random nonce.
  - `POST /api/v1/auth/verify`: Verifies the signed nonce and issues httpOnly cookies.
- **UI Components**:
  - `WalletPickerModal`: Connect wallet interface (supports Freighter).
  - `ConnectButton`: Reusable connect button for nav/login.
  - `login/page.tsx`: The main login page.
- **Middleware**: Added route protection for `/dashboard/*`.
- **Testing**: Added `scripts/test-auth-flow.ts` to verify the E2E auth flow.

## Related Issue
Closes #65

## Verification
1.  Run `npm run dev`.
2.  Go to `http://localhost:3000/login`.
3.  Click "Connect Wallet" -> Select "Freighter".
4.  Approve connection in Freighter.
5.  Sign the message request in Freighter.
6.  Verify redirection to `/dashboard`.
7.  Alternatively, run the test script: `npx tsx scripts/test-auth-flow.ts`.

## Checklist
- [x] Tested manually.
- [x] Added automated test script.
- [x] Linting passed.

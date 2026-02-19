# StelloVault — TypeScript Backend: GitHub Issues

> These issues track building the new Express/TypeScript backend (`/server`) from scratch.
> Each issue maps to a feature group. They are ordered by dependency (implement auth first, etc.).

---

## Issue #1: [SETUP] Initialize Express TypeScript Server Boilerplate

**Labels:** `setup`, `priority: critical`

**Description:**
Set up the foundational Express.js TypeScript project so all other issues can build on top of it.

**Tasks:**
- [x] Create `/server` directory structure: `server/src/controllers`, `server/src/routes`, `server/src/services`, `server/src/middleware`, `server/src/config`
- [x] Add `package.json` with: `express`, `@stellar/stellar-sdk`, `@prisma/client`, `cors`, `helmet`, `morgan`, `dotenv`
- [x] Add `tsconfig.json` targeting ES2020, `outDir: dist`, `rootDir: src`
- [x] Create `.env.example` with all required variables: `PORT`, `DATABASE_URL`, `STELLAR_NETWORK`, `HORIZON_URL`, `RPC_URL`, `FEE_PAYER_SECRET`, contract IDs
- [x] Create `server/src/app.ts`: App factory (CORS, helmet, morgan, JSON parsing, route mounting)
- [ ] Create `server/src/server.ts`: Entry point that calls `app.listen()`
- [ ] Configure graceful shutdown on `SIGTERM`/`SIGINT`

**Acceptance Criteria:**
- `npm run dev` starts the server on port `3001`
- `GET /health` returns `{ status: "ok", database: "connected", version: "..." }`
- TypeScript compiles without errors with `npm run build`

---

## Issue #2: [DATABASE] Define Prisma Schema and Run Migrations

**Labels:** `database`, `priority: critical`

**Description:**
Create the complete Prisma schema with all models needed for StelloVault.

**Tasks:**
- [ ] Create `server/prisma/schema.prisma` with models:
  - `User` — id, stellarAddress (unique), createdAt, updatedAt
  - `Wallet` — id, userId, address, isPrimary, label, verifiedAt
  - `Session` — id, userId, jti (JWT ID), revokedAt
  - `Escrow` — id, buyerId, sellerId, amount, assetCode, status, stellarTxHash, expiresAt, createdAt
  - `Collateral` — id, escrowId, assetCode, amount, metadataHash, status
  - `Loan` — id, borrowerId, lenderId, amount, interestRate, status, dueDate, collateralId
  - `Repayment` — id, loanId, amount, paidAt
  - `OracleEvent` — id, escrowId, oracleAddress, confirmationType, signature, confirmedAt
  - `GovernanceProposal` — id, title, description, proposerId, status, endsAt
  - `GovernanceVote` — id, proposalId, voterAddress, vote, weight
  - `RiskScore` — id, walletAddress, score, components (JSON), recordedAt
- [ ] Run `npx prisma migrate dev --name init`
- [ ] Seed script for local development (`server/prisma/seed.ts`)

**Acceptance Criteria:**
- `npx prisma generate` runs without errors
- `npx prisma migrate dev` applies cleanly
- Prisma Client is importable in service files

---

## Issue #3: [AUTH] Implement Wallet-Based Authentication (Challenge/Sign/Verify)

**Labels:** `auth`, `priority: critical`

**Description:**
Implement the non-custodial, wallet-signature login flow. The backend never holds private keys; it only verifies that a user can sign a nonce with their Stellar keypair.

**Endpoints to implement:**

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/auth/challenge` | Generate a nonce for a wallet address |
| `POST` | `/api/auth/verify` | Verify signed nonce and return JWT tokens |
| `POST` | `/api/auth/refresh` | Refresh access token using a refresh token |
| `POST` | `/api/auth/logout` | Revoke current session (requires auth) |
| `POST` | `/api/auth/logout-all` | Revoke all sessions for current user (requires auth) |
| `GET`  | `/api/auth/me` | Return current authenticated user (requires auth) |

**Detailed Flow:**
1. Client calls `/challenge` with `{ walletAddress }`. Backend saves a time-limited nonce to DB and returns it.
2. Client signs `stellovault:login:<nonce>` with their Stellar private key.
3. Client calls `/verify` with `{ walletAddress, nonce, signature }`. Backend verifies against Stellar public key using `Keypair.verify()`.
4. On success: create or get user, create `Session`, return `{ accessToken, refreshToken }`.

**Implementation Tasks:**
- [ ] `server/src/services/auth.service.ts`
  - `generateChallenge(address: string)` — store nonce in DB, return `{ nonce, expiresAt }`
  - `verifySignature(address, nonce, signature, ip?)` — verify + issue JWT pair
  - `refreshTokens(refreshToken)` — validate + rotate tokens
  - `revokeSession(jti)` — mark session revoked
  - `revokeAllSessions(userId)` — mass revoke
  - `getUserById(userId)` — fetch user profile
  - `getUserWallets(userId)` — return all linked wallets
- [ ] `server/src/controllers/auth.controller.ts` — thin HTTP handlers delegating to service
- [ ] `server/src/routes/auth.routes.ts` — mount endpoints with `authMiddleware` where needed
- [ ] `server/src/config/jwt.ts` — constants for `ACCESS_TOKEN_EXPIRY`, `REFRESH_TOKEN_EXPIRY`
- [ ] `server/src/middleware/auth.middleware.ts` — JWT extraction and verification; populates `req.user`

**Acceptance Criteria:**
- Full challenge → sign → verify flow works end-to-end
- Tokens expire correctly; expired access tokens return `401`
- Revoked sessions are rejected by `authMiddleware`
- `POST /auth/logout-all` returns count of revoked sessions

---

## Issue #4: [WALLET] Implement Wallet Management Endpoints

**Labels:** `wallet`, `auth`

**Description:**
Allow authenticated users to link multiple Stellar wallets, set a primary, update labels, and unlink.

**Endpoints to implement:**

| Method | Path | Description |
|--------|------|-------------|
| `GET`    | `/api/wallets` | List all wallets for current user |
| `POST`   | `/api/wallets/challenge` | Generate challenge for linking a new wallet |
| `POST`   | `/api/wallets` | Link a new wallet (requires signature verification) |
| `DELETE` | `/api/wallets/:id` | Unlink a wallet (cannot remove primary if it's the only one) |
| `PUT`    | `/api/wallets/:id/primary` | Promote a wallet to primary |
| `PATCH`  | `/api/wallets/:id` | Update wallet label |

**Implementation Tasks:**
- [ ] Extend `server/src/services/auth.service.ts`:
  - `linkWallet(userId, address, nonce, signature, label?)` — verify + persist wallet
  - `unlinkWallet(userId, walletId)` — guard against removing last wallet
  - `setPrimaryWallet(userId, walletId)` — swap primary flag
- [ ] `server/src/controllers/wallet.controller.ts`
- [ ] `server/src/routes/wallet.routes.ts` (all routes require `authMiddleware`)

**Acceptance Criteria:**
- Cannot unlink the only/primary wallet — returns `400 Bad Request`
- Cannot link duplicate wallet address — returns `409 Conflict`
- Signature verification reuses the same nonce flow as `auth.service.ts`

---

## Issue #5: [USER] Implement User Profile Endpoints

**Labels:** `user`

**Description:**
Expose CRUD endpoints for user profile management.

**Endpoints to implement:**

| Method | Path | Description |
|--------|------|-------------|
| `GET`  | `/api/users/:id` | Get a user profile by ID |
| `POST` | `/api/users` | Create a user (internal/admin use) |

**Implementation Tasks:**
- [ ] `server/src/services/user.service.ts`:
  - `getUserById(id)` — fetch from DB including primary wallet
  - `createUser(stellarAddress)` — idempotent upsert on stella address
- [ ] `server/src/controllers/user.controller.ts`
- [ ] `server/src/routes/user.routes.ts`

**Acceptance Criteria:**
- `GET /api/users/:id` returns `404` for unknown IDs
- Creating a duplicate address returns the existing user (upsert behavior)

---

## Issue #6: [ESCROW] Implement Escrow Service and Endpoints

**Labels:** `escrow`, `soroban`, `priority: high`

**Description:**
Core escrow lifecycle: creation (with XDR building for Soroban), listing, retrieval, webhook updates, and WebSocket broadcasting on state changes.

**Endpoints to implement:**

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/escrows` | Create a new escrow (returns unsigned XDR) |
| `GET`  | `/api/escrows` | List escrows (filter by buyer, seller, status; paginate) |
| `GET`  | `/api/escrows/:id` | Get a single escrow by UUID |
| `POST` | `/api/escrows/webhook` | Receive on-chain status update (secured with `X-Webhook-Secret`) |

**Escrow Lifecycle States:** `PENDING → ACTIVE → COMPLETED | DISPUTED | EXPIRED`

**Implementation Tasks:**
- [ ] `server/src/services/escrow.service.ts`:
  - `createEscrow(req)` — Build Soroban XDR via `ContractService` with Fee Payer, persist to DB, return `{ escrowId, xdr }`
  - `getEscrow(id)` — DB fetch
  - `listEscrows(query)` — Filtered, paginated query via Prisma
  - `processEscrowEvent(event)` — Update DB status from webhook; emit WebSocket broadcast
  - `timeoutDetector()` — Background job; queries for `ACTIVE` escrows past `expiresAt`, marks them `EXPIRED`
- [ ] `server/src/controllers/escrow.controller.ts`
- [ ] `server/src/routes/escrow.routes.ts`
- [ ] Webhook validation: reject if `X-Webhook-Secret` header doesn't match `WEBHOOK_SECRET` env var; reject with `503` if secret not configured
- [ ] WebSocket broadcast: on status change, emit `{ type: "ESCROW_UPDATED", escrowId, status }` to all connected clients

**Acceptance Criteria:**
- `POST /api/escrows` returns `{ escrowId, xdr }` where `xdr` is a base64-encoded unsigned transaction XDR
- Unauthenticated webhook returns `401`; misconfigured webhook returns `503`
- Background timeout job runs every 60 seconds

---

## Issue #7: [COLLATERAL] Implement Collateral Service and Endpoints

**Labels:** `collateral`, `soroban`

**Description:**
Manage collateral records linked to escrows, including on-chain indexing of collateral events.

**Endpoints to implement:**

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/collateral` | Register a new collateral record |
| `GET`  | `/api/collateral` | List all collateral records (filter by escrow, status) |
| `GET`  | `/api/collateral/:id` | Get single collateral by UUID |
| `GET`  | `/api/collateral/metadata/:hash` | Lookup collateral by metadata hash |

**Implementation Tasks:**
- [ ] `server/src/services/collateral.service.ts`:
  - `createCollateral(escrowId, assetCode, amount, metadataHash)` — persist record
  - `getCollateralById(id)` — DB fetch
  - `getCollateralByMetadataHash(hash)` — DB lookup
  - `listCollateral(query)` — Prisma filtered query
  - `startIndexer()` — Background polling of Soroban RPC for `CollateralDeposited` events; update DB on match
- [ ] `server/src/controllers/collateral.controller.ts`
- [ ] `server/src/routes/collateral.routes.ts`

**Acceptance Criteria:**
- Collateral lookup by metadata hash returns `404` if not found
- Indexer runs in the background and updates collateral status when on-chain events are detected

---

## Issue #8: [LOAN] Implement Loan Service and Endpoints

**Labels:** `loan`, `soroban`, `priority: high`

**Description:**
Loan lifecycle management: issuance, repayment tracking, and listing.

**Endpoints to implement:**

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/loans` | Issue a new loan (builds XDR for Soroban call) |
| `GET`  | `/api/loans` | List loans (filter by `borrowerId`, `lenderId`, `status`) |
| `GET`  | `/api/loans/:id` | Get a single loan |
| `POST` | `/api/loans/repay` | Record a repayment |

**Implementation Tasks:**
- [ ] `server/src/services/loan.service.ts`:
  - `issueLoan(req)` — validate collateral ratio, build Soroban XDR, persist loan record as `PENDING`
  - `getLoan(id)` — DB fetch with relations
  - `listLoans(borrowerId?, lenderId?, status?)` — Prisma filtered query
  - `recordRepayment(req)` — persist repayment, check if loan is fully paid, update status to `REPAID`
- [ ] `server/src/controllers/loan.controller.ts`
- [ ] `server/src/routes/loan.routes.ts`

**Acceptance Criteria:**
- `POST /api/loans` returns an unsigned XDR the client must sign
- Repayment recording checks outstanding balance and auto-closes fully repaid loans
- Listing supports `status` filter with values: `PENDING`, `ACTIVE`, `REPAID`, `DEFAULTED`

---

## Issue #9: [ORACLE] Implement Oracle Confirmation Service and Endpoints

**Labels:** `oracle`, `priority: high`

**Description:**
Oracle nodes submit cryptographic confirmations for on-chain events. The backend validates signatures, deduplicates, and flags disputes.

**Endpoints to implement:**

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/oracles` | Register a new oracle node |
| `GET`  | `/api/oracles` | List active oracle nodes |
| `GET`  | `/api/oracles/:address` | Get oracle by Stellar address |
| `POST` | `/api/oracles/:address/deactivate` | Deactivate an oracle |
| `POST` | `/api/confirmations` | Submit an oracle confirmation |
| `GET`  | `/api/confirmations/:escrowId` | Get all confirmations for an escrow |
| `GET`  | `/api/oracles/metrics` | Get oracle network health metrics |
| `POST` | `/api/oracle/dispute` | Flag an escrow event for dispute |

**Implementation Tasks:**
- [ ] `server/src/services/oracle.service.ts`:
  - `registerOracle(address)` — persist oracle node entry
  - `deactivateOracle(address)` — soft-delete oracle
  - `confirmOracleEvent(req)` — verify signature, check duplicates (rate limit), persist confirmation
  - `listOracleEvents(query)` — filtered listing
  - `getOracleEvent(id)` — fetch by UUID
  - `flagDispute(escrowId, reason, disputerAddress)` — mark escrow as disputed
  - `getOracleMetrics()` — compute uptime, confirmation rates, active nodes
- [ ] `server/src/controllers/oracle.controller.ts`
- [ ] `server/src/routes/oracle.routes.ts`

**Error Handling:**
- Duplicate confirmation → `409 Conflict`
- Rate-limited oracle → `429 Too Many Requests`
- Invalid oracle signature → `401 Unauthorized`
- Empty dispute reason → `400 Bad Request`

**Acceptance Criteria:**
- Oracle nodes are identified by Stellar address
- Confirmation requires valid Stellar signature
- Dispute endpoint changes escrow status to `DISPUTED` and broadcasts WS event

---

## Issue #10: [GOVERNANCE] Implement Governance Service and Endpoints

**Labels:** `governance`, `soroban`

**Description:**
On-chain DAO governance: create proposals, vote, and audit the results.

**Endpoints to implement:**

| Method | Path | Description |
|--------|------|-------------|
| `GET`  | `/api/governance/proposals` | List all proposals |
| `POST` | `/api/governance/proposals` | Create a new governance proposal |
| `GET`  | `/api/governance/proposals/:id` | Get single proposal |
| `GET`  | `/api/governance/proposals/:id/votes` | Get all votes for a proposal |
| `POST` | `/api/governance/votes` | Cast a vote on a proposal |
| `GET`  | `/api/governance/metrics` | Protocol governance health metrics |
| `GET`  | `/api/governance/parameters` | Current on-chain governance parameters |
| `GET`  | `/api/governance/audit` | Audit log of all governance actions |

**Implementation Tasks:**
- [ ] `server/src/services/governance.service.ts`:
  - `getProposals()` — list from DB
  - `createProposal(req)` — build Soroban XDR for on-chain proposal; persist off-chain record
  - `getProposalById(id)` — DB fetch
  - `getProposalVotes(proposalId)` — fetch votes with voter addresses and weights
  - `submitVote(req)` — verify voter has quorum weight; persist vote; build contract call XDR
  - `getMetrics()` — count proposals, participation rate, avg vote weight
  - `getParameters()` — read on-chain governance parameters via Soroban `simulateCall`
  - `getAuditLog()` — paginated log of all on-chain governance events from indexer
- [ ] `server/src/controllers/governance.controller.ts`
- [ ] `server/src/routes/governance.routes.ts`

**Acceptance Criteria:**
- Proposals have statuses: `OPEN`, `PASSED`, `REJECTED`, `EXECUTED`
- Voting requires valid auth; duplicate votes return `409`
- Audit log is sourced from `EventMonitoringService`

---

## Issue #11: [RISK] Implement Risk Scoring Engine and Endpoints

**Labels:** `risk`, `analytics`, `priority: high`

**Description:**
Stateless risk scoring for any Stellar wallet based on on-chain transaction history, collateral ratios, and defaults. Supports historical backtesting and simulation.

**Endpoints to implement:**

| Method | Path | Description |
|--------|------|-------------|
| `GET`  | `/api/risk/:wallet` | Compute current risk score for a wallet |
| `GET`  | `/api/risk/:wallet/history` | Historical scores (supports `?start_date=&end_date=`) |
| `POST` | `/api/risk/:wallet/simulate` | Simulate score impact of a hypothetical action |

**Risk Score Components (all weighted):**
- Transaction history on Stellar (volume, frequency, age)
- Loan repayment history (on-time vs defaulted)
- Collateral coverage ratio
- Escrow dispute history

**Implementation Tasks:**
- [ ] `server/src/services/risk-engine.service.ts`:
  - `calculateRiskScore(wallet)` — fetch Horizon transaction history + DB records; compute weighted score; persist `RiskScore` snapshot; return `{ score, components, grade }`
  - `getHistoricalScores(wallet, startDate, endDate)` — query `RiskScore` table with date range
  - `simulateScoreImpact(wallet, scenario)` — compute score with hypothetical loan/collateral change applied; return `{ currentScore, projectedScore, delta }`
- [ ] `server/src/controllers/risk.controller.ts`
- [ ] `server/src/routes/risk.routes.ts`

**Types to define:**
```typescript
type RiskScoreResponse = {
  wallet: string;
  score: number;       // 0–1000
  grade: 'A' | 'B' | 'C' | 'D' | 'F';
  components: {
    transactionHistory: number;
    repaymentRecord: number;
    collateralCoverage: number;
    disputeHistory: number;
  };
  computedAt: Date;
};
```

**Acceptance Criteria:**
- Score is a value between 0–1000
- `GET /api/risk/:wallet/history` filters by date range correctly
- `POST /api/risk/:wallet/simulate` doesn't persist the simulated score

---

## Issue #12: [ANALYTICS] Implement Analytics Endpoint

**Labels:** `analytics`

**Description:**
Return aggregated platform metrics for dashboards.

**Endpoints to implement:**

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/analytics` | Return platform-wide statistics |

**Data to aggregate:**
- `totalEscrows`, `activeEscrows`, `completedEscrows`, `disputedEscrows`
- `totalLoans`, `activeLoans`, `defaultedLoans`
- `totalVolumeUSDC`
- `totalUsers`, `activeWallets`
- `governanceProposals`, `participationRate`

**Implementation Tasks:**
- [ ] `server/src/services/analytics.service.ts`:
  - `getPlatformStats()` — run Prisma aggregate queries; combine results
- [ ] `server/src/controllers/analytics.controller.ts`
- [ ] `server/src/routes/analytics.routes.ts`

**Acceptance Criteria:**
- Response is a flat JSON object with all stats
- Query is cached for 60 seconds to avoid repeated DB aggregations

---

## Issue #13: [EVENTS] Implement Soroban Event Indexer Service

**Labels:** `indexer`, `soroban`, `background`

**Description:**
Background long-running job that polls the Soroban RPC for contract events and updates the off-chain DB to mirror on-chain state. This bridges the chain → DB synchronization.

**Contracts to monitor:**
- `COLLATERAL_CONTRACT_ID` → `CollateralDeposited`, `CollateralReleased`
- `ESCROW_CONTRACT_ID` → `EscrowCreated`, `EscrowSettled`, `EscrowExpired`
- `LOAN_CONTRACT_ID` → `LoanIssued`, `LoanRepaid`, `LoanDefaulted`

**Implementation Tasks:**
- [ ] `server/src/services/event-monitoring.service.ts`:
  - `start()` — main loop polling every 10s; track last processed ledger in DB
  - `processCollateralEvents(events)` — update Collateral records
  - `processEscrowEvents(events)` — update Escrow records + broadcast via WS
  - `processLoanEvents(events)` — update Loan status
  - `processGovernanceEvents(events)` — append to audit log
- [ ] `server/src/config/contracts.ts` — contract IDs loaded from env
- [ ] Store last processed ledger number in a `IndexerState` table or file

**Acceptance Criteria:**
- Indexer starts automatically when the server boots
- On-chain events update DB within 2 polling cycles
- Ledger cursor is persisted so restarts don't reprocess old events

---

## Issue #14: [WEBSOCKET] Implement Real-time WebSocket Support

**Labels:** `websocket`, `realtime`

**Description:**
Push real-time updates to connected frontends when escrow/loan/governance states change.

**Events to broadcast:**
- `ESCROW_CREATED` `{ escrowId, buyerId, sellerId }`
- `ESCROW_UPDATED` `{ escrowId, status }`
- `LOAN_UPDATED` `{ loanId, status }`
- `GOVERNANCE_VOTE_CAST` `{ proposalId, newTally }`

**Implementation Tasks:**
- [ ] Install `ws` package
- [ ] `server/src/services/websocket.service.ts`:
  - `WsState` class: manage connected client set
  - `broadcastEvent(event)` — serialize + send to all active connections
  - Handle client ping/pong to detect disconnects
- [ ] Mount WebSocket endpoint at `GET /ws` in `server/src/app.ts`
- [ ] Wire `WsState` into `EscrowService`, `OracleService`, and `GovernanceService`

**Acceptance Criteria:**
- Clients connect to `ws://localhost:3001/ws`
- Events are broadcast within one second of DB update
- Stale connections are cleaned up automatically

---

## Issue #15: [MIDDLEWARE] Implement Core Middleware

**Labels:** `middleware`, `priority: critical`

**Description:**
Provide security, rate limiting, request tracing, and error handling middleware.

**Tasks:**
- [ ] `server/src/middleware/auth.middleware.ts` — extract bearer JWT, verify, attach `req.user`
- [ ] `server/src/middleware/rate-limit.middleware.ts` — express-rate-limit, 100 req/min per IP
- [ ] `server/src/middleware/error.middleware.ts` — centralized error handler that maps error types to HTTP status codes:
  - `NotFoundError` → 404
  - `UnauthorizedError` → 401
  - `ConflictError` → 409
  - `ValidationError` → 400
  - All others → 500
- [ ] `server/src/middleware/request-trace.middleware.ts` — log method, path, status, duration
- [ ] `server/src/config/errors.ts` — define custom error classes inheriting from `Error`

**Acceptance Criteria:**
- All errors return `{ success: false, error: "message" }` JSON body
- Rate limit returns `429` with `Retry-After` header
- All requests are logged with status code on completion

---

## Issue #16: [CONTRACT-SERVICE] Implement Soroban Contract Service (XDR Builder)

**Labels:** `soroban`, `blockchain`, `priority: critical`

**Description:**
The `ContractService` is the core of account abstraction. It builds transaction XDRs with the backend as Fee Payer, ready for client-side signing.

**Implementation Tasks:**
- [ ] `server/src/services/contract.service.ts`:
  - `buildContractInvokeXDR(contractId, method, args, sourcePublicKey)`:
    - Load source account from Horizon
    - Build `TransactionBuilder` with Fee Payer (`FEE_PAYER_PUBLIC`) as the outer source
    - Create contract invocation operation using `Contract.call()`
    - Return base64-encoded XDR
  - `simulateCall(contractId, method, args)` — call Soroban RPC `simulateTransaction`; decode result with `scValToNative`
  - `submitXDR(signedXDR)` — submit a fully signed XDR to the network
- [ ] `server/src/services/blockchain.service.ts`:
  - `getAccountBalance(address, assetCode)` — load account from Horizon
  - `buildNativePayment(from, to, amount)` — XLM payment (for covering minimum reserves)

**Acceptance Criteria:**
- Returned XDR is valid and can be decoded by `stellar-sdk`
- Fee Payer key signs the outer transaction; user key signs the auth entries
- `simulateCall` correctly decodes struct `ScVal` return types

---

## Implementation Order

```
#1 Setup → #2 Database → #15 Middleware → #3 Auth → #4 Wallet → #5 User
    → #16 Contract Service → #6 Escrow → #7 Collateral → #8 Loan
    → #9 Oracle → #10 Governance → #11 Risk → #12 Analytics
    → #13 Indexer → #14 WebSocket
```

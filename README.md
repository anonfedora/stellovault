# StelloVault

**A secure trade finance dApp built on Stellar & Soroban**  
Tokenizing collateral (invoices, commodities, etc.) to unlock instant liquidity for exporters and importers, bridging the massive trade finance gap.

[![Stellar](https://img.shields.io/badge/Built%20on-Stellar-blue?logo=stellar)](https://stellar.org)
[![Soroban](https://img.shields.io/badge/Smart%20Contracts-Soroban-orange)](https://soroban.stellar.org)
[![Next.js](https://img.shields.io/badge/Frontend-Next.js-black?logo=next.js)](https://nextjs.org)
[![TypeScript](https://img.shields.io/badge/Backend-TypeScript-3178C6?logo=typescript)](https://www.typescriptlang.org)
[![CI](https://github.com/anonfedora/stellovault/actions/workflows/ci.yml/badge.svg)](https://github.com/anonfedora/stellovault/actions/workflows/ci.yml)

---

## 🚀 Overview

StelloVault is a trade finance dApp that enables SMEs to tokenize real-world assets (invoices, commodities) as Stellar assets with embedded metadata, use them as collateral in multi-signature escrows managed by **Soroban smart contracts**, and unlock instant cross-border liquidity.

Key innovations:
- **Collateral Tokenization** — Real assets become fractional, traceable Stellar tokens.
- **Automated Escrows** — Multi-sig + conditional release triggered by shipment verification oracles.
- **Dynamic Financing** — Algorithmic loans based on on-chain history and utilization.
- **Risk Scoring** — On-chain creditworthiness scoring using transaction history.
- **Governance** — Quadratic voting for protocol parameters and accepted collateral types.

> **Trade finance gap:** $100–120B+ annually (Afreximbank, African Development Bank), disproportionately affecting SMEs under the AfCFTA. StelloVault targets reducing intermediary costs by up to **50%**.

---

## ✨ Key Features

| Feature | Description |
|---------|-------------|
| Collateral Tokenization | Mint Stellar assets from invoices/goods with provenance metadata |
| Multi-Sig Escrows | Soroban enforces release on oracle confirmation |
| Oracle Integration | Real-time data feeds for shipment & quality verification |
| Risk Scoring Engine | On-chain history → dynamic loan terms |
| Frontend Dashboard | Next.js UI for deal origination, escrow monitoring, repayments |
| Governance Module | On-chain quadratic voting for protocol parameters |
| Flash Settlements | Instant cross-border payments via Stellar DEX/path payments |
| Real-time Updates | WebSocket push for escrow/loan state changes |

---

## 📂 Repository Structure

```
stellovault/
├── contracts/          # Soroban Smart Contracts (Rust)
├── frontend/           # Next.js Frontend Application
├── server/             # TypeScript/Express Backend API  ← active
├── backend/            # Rust/Axum Backend  (archived — superseded by /server)
└── .github/
    └── workflows/
        └── ci.yml      # CI: server (TS) + contracts (Rust)
```

### `contracts/` — Soroban Smart Contracts (Rust)

- **Tech:** Rust · Soroban SDK
- **Purpose:** On-chain escrow, collateral tokenization, governance, fee management
- **Build:** `cargo build --release --target wasm32-unknown-unknown`

### `frontend/` — User Interface

- **Tech:** Next.js 14+, TypeScript, Tailwind CSS
- **Features:** Dashboard, escrow management, collateral upload, governance voting
- **Dev:** `npm run dev`

### `server/` — TypeScript Backend API *(active backend)*

- **Tech:** Express.js · TypeScript · Prisma · PostgreSQL · `@stellar/stellar-sdk`
- **Pattern:** Non-custodial · Account Abstraction (Fee Payer) · Event-driven
- **Dev:** `npm run dev`

```
server/
├── prisma/
│   └── schema.prisma               # DB models (User, Loan, Escrow, Collateral …)
├── src/
│   ├── app.ts                      # Express app entry point
│   ├── config/
│   │   ├── env.ts                  # Typed environment variables
│   │   ├── contracts.ts            # Soroban contract IDs
│   │   └── errors.ts               # Custom error classes
│   ├── controllers/                # HTTP request handlers (thin orchestration layer)
│   │   ├── auth.controller.ts
│   │   ├── wallet.controller.ts
│   │   ├── user.controller.ts
│   │   ├── escrow.controller.ts
│   │   ├── collateral.controller.ts
│   │   ├── loan.controller.ts
│   │   ├── oracle.controller.ts
│   │   ├── governance.controller.ts
│   │   ├── risk.controller.ts
│   │   └── user.controller.ts      # also handles /analytics
│   ├── routes/                     # Express routers mounted under /api/v1
│   ├── services/                   # Core business & blockchain logic
│   │   ├── blockchain.service.ts   # Horizon / native Stellar ops
│   │   ├── contract.service.ts     # Soroban XDR builder (Account Abstraction)
│   │   ├── database.service.ts     # Prisma ORM wrappers
│   │   └── event-monitoring.service.ts  # On-chain event poller
│   └── middleware/
│       ├── auth.middleware.ts      # JWT Bearer verification
│       ├── error.middleware.ts     # Central error → HTTP status mapping
│       └── rate-limit.middleware.ts
├── package.json
├── tsconfig.json
├── .env.example
└── migration_issues.md             # GitHub issues for full feature implementation
```

**API Routes (all under `/api/v1`):**

| Prefix | Domain |
|--------|--------|
| `/auth` | Wallet challenge/sign/verify, JWT rotation |
| `/wallets` | Link, unlink, set-primary wallet |
| `/users` | User profiles |
| `/escrows` | Escrow lifecycle + webhook |
| `/collateral` | Collateral records |
| `/loans` | Loan issuance + repayments |
| `/oracles` | Oracle node registry + confirmations |
| `/confirmations` | Oracle event confirmations |
| `/governance` | Proposals, votes, audit log |
| `/risk` | Risk scoring + historical + simulation |
| `/analytics` | Platform-wide aggregated stats |

### `backend/` — Rust/Axum Backend *(archived)*

The original Rust backend is preserved here for reference. It has been superseded by the TypeScript server above. See `backend/README.md` for details.

---

## 🛠 Getting Started

### Prerequisites

| Tool | Version |
|------|---------|
| Node.js | 20+ |
| npm | 9+ |
| PostgreSQL | 14+ |
| Rust | stable |
| Soroban CLI | latest |

### TypeScript Server

```bash
cd server
cp .env.example .env   # fill in DATABASE_URL, FEE_PAYER_SECRET, contract IDs
npm install
npx prisma migrate dev
npm run dev            # starts on http://localhost:3001
```

### Soroban Contracts

```bash
cd contracts
cargo build --release --target wasm32-unknown-unknown
cargo test
```

### Frontend

```bash
cd frontend
npm install
npm run dev            # starts on http://localhost:3000
```

---

## 🔄 Transaction Flow (Non-Custodial / Account Abstraction)

```
User                    Server                    Stellar/Soroban
 │                         │                            │
 │  POST /api/v1/escrows   │                            │
 │────────────────────────▶│                            │
 │                         │  Build XDR (Fee Payer src) │
 │                         │───────────────────────────▶│
 │  { escrowId, xdr }      │                            │
 │◀────────────────────────│                            │
 │                         │                            │
 │  Sign auth entries      │                            │
 │  (client-side only)     │                            │
 │────────────────────────▶│                            │
 │                         │  Sign as Fee Payer         │
 │                         │  Submit signed XDR         │
 │                         │───────────────────────────▶│
 │                         │         { txHash }         │
 │  { success, txHash }    │                            │
 │◀────────────────────────│                            │
```

The backend **never holds user private keys**. It only acts as Fee Payer for sponsoring gas costs.

---

## ⚙️ CI

Two parallel jobs run on every push/PR to `main` or `contract`:

| Job | Directory | Checks |
|-----|-----------|--------|
| `server` | `./server` | `npm ci` → `prisma generate` → `tsc --noEmit` → `npm test` |
| `contracts` | `./contracts` | `cargo fmt` → `cargo clippy` → `cargo build` → `cargo test` |

---

## 🤝 Contributing

1. Fork → clone → `git remote add upstream https://github.com/anonfedora/stellovault.git`
2. `git checkout -b feature/my-feature`
3. Implement + test
4. `git push origin feature/my-feature` → open a PR against `main`

**Guidelines:**
- TypeScript: follow `prettier` formatting (`server/`)
- Rust: `cargo fmt` + `cargo clippy` must pass (`contracts/`)
- All new endpoints need controller + route + service
- Use the issue tracker — see `server/migration_issues.md` for the full feature backlog

---

## 📄 License

MIT © StelloVault Contributors

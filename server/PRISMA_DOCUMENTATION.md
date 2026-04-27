# Prisma Documentation in StelloVault Server

## Overview
The StelloVault server uses Prisma as its ORM (Object-Relational Mapping) tool for interacting with a PostgreSQL database. The Prisma setup includes schema definition, client configuration, and migration management.

## Files Related to Prisma

### 1. Prisma Client Configuration (`server/src/config/prisma.ts`)
- Sets up a PrismaClient instance with a custom PostgreSQL adapter (`@prisma/adapter-pg`)
- Implements a singleton pattern to prevent multiple instances in development (using globalThis)
- Configures logging based on environment (query, error, warn in development; only error in production)
- Exports a single `prisma` instance for use throughout the application

Key aspects:
- Uses `DATABASE_URL` environment variable for connection string
- Requires `@prisma/adapter-pg` for Prisma 7+ compatibility with PostgreSQL
- Prevents hot-reload issues in development by storing instance in global scope

### 2. Prisma Schema (`server/prisma/schema.prisma`)
- Defines the database models and relationships
- Uses PostgreSQL as the provider
- Generates Prisma client to `../src/generated/prisma`

#### Major Models Defined:
- **User**: Stellar address, name, role, with relationships to wallets, challenges, escrows, investments, sessions, loans, and governance proposals
- **Wallet**: Belongs to user, stores Stellar address and verification status
- **Session**: JWT sessions with JTI and revocation tracking
- **WalletChallenge**: For authentication and wallet linking
- **Escrow**: Core escrow functionality with buyer/seller relationships, collateral, and oracle events
- **Collateral**: Assets locked in escrow
- **Loan**: Lending/borrowing with interest rates and repayment schedules
- **PaymentSession & Payment**: For processing loan repayments
- **Investment**: User investments in the platform
- **Oracle & OracleConfirmation**: For external data verification
- **Dispute**: For resolving escrow disagreements
- **GovernanceProposal & GovernanceVote**: Platform governance system
- **RiskScore & AuditLog**: Risk assessment and activity logging

#### Enums Defined:
- PaymentStatus, Role, ChallengePurpose, EscrowStatus, CollateralStatus, LoanStatus, PaymentSessionStatus, GovernanceStatus, DisputeStatus

### 3. Package Configuration (`server/package.json`)
#### Dependencies:
- `@prisma/client`: ^7.4.1 (Prisma client generator)
- `@prisma/adapter-pg`: ^7.4.1 (PostgreSQL adapter for Prisma 7)
- `@prisma/config`: ^7.4.0
- `pg`: ^8.20.0 (PostgreSQL client)

#### Dev Dependencies:
- `prisma`: ^7.4.1 (Prisma CLI)
- `ts-node`: ^10.9.2 (for running TypeScript scripts)

#### Scripts:
- `prisma:generate`: `prisma generate` (generates Prisma client)
- `prisma:migrate`: `prisma migrate dev` (runs migrations)
- Seed script: `ts-node prisma/seed.ts` (defined in prisma section)

### 4. Prisma Migrations (`server/prisma/migrations/`)
- Contains SQL migration files for schema evolution
- Migration history includes:
  - Initial unified schema
  - Risk score additions
  - Escrow status updates
  - Payment sessions and models
  - RBAC (Role-Based Access Control) models
  - Various other schema updates

### 5. Prisma Seed (`server/prisma/seed.ts`)
- Seed data for initial database population
- Likely creates initial roles, permissions, and admin users

## Usage Throughout Codebase
The Prisma client is imported and used in various services and controllers:
- Imported as `import { prisma } from "../config/prisma";`
- Used in services like auth.service.ts, escrow.service.ts, loan.service.ts, etc.
- For database operations: finding, creating, updating, and deleting records

## Key Features of Implementation
1. **Custom Adapter**: Uses `@prisma/adapter-pg` for direct PostgreSQL connection pooling
2. **Singleton Pattern**: Prevents multiple Prisma instances during development hot-reloads
3. **Environment-Based Logging**: Detailed logging in development, minimal in production
4. **Modular Schema**: Well-organized schema with clear sections and comments
5. **Relationship Modeling**: Extensive use of Prisma's relation features for complex data models
6. **Indexing**: Strategic indexes on frequently queried fields
7. **Enum Usage**: Prisma enums for type-safe status fields

## Potential Improvements
1. Consider moving to Prisma's native PostgreSQL driver if adapter limitations arise
2. Add more detailed documentation in schema comments
3. Consider using Prisma's preview features for enhanced functionality
4. Add Prisma Studio to development workflow for easier database inspection
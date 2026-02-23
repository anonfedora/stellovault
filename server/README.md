# StelloVault — Server

TypeScript/Express backend for StelloVault. Uses PostgreSQL via Prisma ORM, JWT authentication, and Stellar SDK for on-chain interactions.

---

## Prerequisites

| Tool | Version |
|---|---|
| Node.js | ≥ 18 |
| npm | ≥ 9 |
| PostgreSQL | ≥ 14 (running locally or via Docker) |

---

## Setup

### 1. Install dependencies

```bash
npm install
```

### 2. Configure environment

```bash
cp .env.example .env
```

Edit `.env` and set your `DATABASE_URL`:

```env
DATABASE_URL="postgresql://<user>:<password>@localhost:5432/stellovault?schema=public"
```

> **Tip — quick local Postgres with Docker:**
> ```bash
> docker run -d \
>   --name stellovault-db \
>   -e POSTGRES_USER=stella \
>   -e POSTGRES_PASSWORD=secret \
>   -e POSTGRES_DB=stellovault \
>   -p 5432:5432 \
>   postgres:16-alpine
> ```
> Then set `DATABASE_URL="postgresql://stella:secret@localhost:5432/stellovault?schema=public"`

---

## Database Setup (Prisma)

### First-time setup

Run the migration to create all tables, then seed with local dev data:

```bash
npx prisma migrate dev --name init
npx prisma db seed
```

### Regenerate the Prisma Client

Run this whenever you modify `prisma/schema.prisma`:

```bash
npx prisma generate
```

### Reset the database (wipe all data and re-run migrations)

```bash
npx prisma migrate reset
```

> ⚠️  This **deletes all data**. Only use in local development.

### Apply pending migrations (CI / production)

```bash
npx prisma migrate deploy
```

### Open Prisma Studio (GUI data browser)

```bash
npx prisma studio
```

---

## Running the Dev Server

```bash
npm run dev
```

Server starts on **http://localhost:3001**. Health check: `GET /health`

---

## Prisma Quick Reference

| Command | Description |
|---|---|
| `npx prisma migrate dev --name <name>` | Create a new migration from schema changes |
| `npx prisma migrate reset` | Drop DB, re-run all migrations, re-seed |
| `npx prisma migrate deploy` | Apply pending migrations (non-interactive) |
| `npx prisma generate` | Regenerate the Prisma Client |
| `npx prisma db seed` | Run `prisma/seed.ts` to populate dev data |
| `npx prisma studio` | Open visual DB browser at http://localhost:5555 |
| `npx prisma format` | Auto-format `schema.prisma` |
| `npx prisma validate` | Validate the schema without running migrations |

---

## Project Structure

```
server/
├── prisma/
│   ├── schema.prisma      # Prisma data model
│   ├── seed.ts            # Local dev seed data
│   └── migrations/        # Auto-generated migration SQL
├── src/
│   ├── app.ts             # Express app factory
│   ├── config/            # JWT, contracts, environment config
│   ├── controllers/       # HTTP handlers (thin layer)
│   ├── middleware/        # Auth, rate-limit, error handling
│   ├── routes/            # Route mounting
│   └── services/          # Business logic
├── .env.example
├── package.json
└── tsconfig.json
```

---

## Scripts

| Script | Command |
|---|---|
| Start (production) | `npm start` |
| Dev (watch mode) | `npm run dev` |
| Build | `npm run build` |
| Generate Prisma Client | `npm run prisma:generate` |
| Run migrations | `npm run prisma:migrate` |

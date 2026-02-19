# StelloVault Server

Rust backend server for StelloVault - a decentralized escrow platform built on Stellar/Soroban.

## Features

- **Escrow Management** - Create, track, and manage trade escrows
- **Real-time Updates** - WebSocket support for live escrow events
- **Blockchain Integration** - Soroban smart contract interaction (simulated)
- **Rate Limiting** - Token bucket rate limiting per IP
- **Security Headers** - XSS, clickjacking, and MIME sniffing protection

## Project Structure

```
src/
├── config/       # Environment-based configuration
├── db/           # Database pool & migrations
├── error/        # Centralized API error handling
├── escrow/       # Escrow domain (model, service, event_listener)
├── handlers/     # HTTP request handlers
├── middleware/   # Request tracing, rate limiting, security
├── models/       # Shared data models
├── routes/       # Route definitions
├── services/     # Business logic services
├── state/        # Application state
├── websocket/    # WebSocket handling
├── main.rs       # Application entry point
└── lib.rs        # Library exports
```

## Prerequisites

- Rust 1.70+
- Docker & Docker Compose
- PostgreSQL 15 (via Docker)

## Quick Start

1. **Start the database**:
   ```bash
   docker compose up -d
   ```

2. **Configure environment**:
   ```bash
   cp .env.example .env
   # Edit .env as needed
   ```

3. **Run the server**:
   ```bash
   cargo run
   ```

4. **Verify it's working**:
   ```bash
   curl http://localhost:3001/health
   ```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `ENVIRONMENT` | dev/staging/prod | `dev` |
| `PORT` | Server port | `3001` |
| `RUST_LOG` | Log level | `info` |
| `RATE_LIMIT_RPS` | Requests per second per IP | `100` |
| `CORS_ALLOWED_ORIGINS` | Comma-separated origins | Permissive |
| `HORIZON_URL` | Stellar Horizon API | Testnet |
| `SOROBAN_RPC_URL` | Soroban RPC endpoint | Testnet |

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check with DB status |
| GET | `/ws` | WebSocket connection |
| GET | `/api/escrows` | List escrows |
| POST | `/api/escrows` | Create escrow |
| GET | `/api/escrows/:id` | Get escrow by ID |
| POST | `/api/escrows/webhook` | Webhook for status updates |
| GET | `/api/users/:id` | Get user |
| POST | `/api/users` | Create user |
| GET | `/api/analytics` | Get analytics |

## Development

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Build for release
cargo build --release
```

## Database Migrations

Migrations are in `migrations/` and run automatically on startup. To create the required tables, ensure your migration files are uncommented and the database is accessible.

## License

MIT

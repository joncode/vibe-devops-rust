# 🚀 Vibe DevOps Server

**Ship your vibe-coded app with production-grade infrastructure.**

You built something cool with AI in 20 minutes. Now you need auth, database, security, WebSockets, and an admin panel. Look no further, you are in the right place.

## Why This Exists

Vibe coding is fast. Production code is slow. This repo bridges the gap — giving you battle-tested infrastructure so you can focus on the fun stuff.

**What's included:**
- 🚀 **Fast** - Built with Rust and Axum for maximum performance
- 🔐 **Auth** — JWT + refresh tokens, Argon2 password hashing, session management
- 🗄️ **Database** — PostgreSQL with compile-time checked queries (SQLx)
- ⚡ **Redis** — Sessions, caching, rate limiting
- ⚡ **Caching** - Redis for sessions and caching
- 🔌 **WebSocket** — Real-time wss: infrastructure baked in
- 🛡️ **Security** — CORS, rate limiting, input validation, proper error handling
- 📊 **Admin Panel** — See your data, paginated, no extra setup, separate user login
- 🐳 **Docker Ready** - Docker Compose for easy development setup
- 🏥 **Health Checks** - Kubernetes-ready liveness and readiness probes

## The Philosophy

Your AI-generated frontend is fire. Your backend is a liability.

This isn't a framework. It's the production code you'd write if you had 3 months and a security audit. Fork it, plug in your app, ship it.  Let me know in the comments how you like it. 

Built in Rust because fast, safe, and you shouldn't have to think about it.
A production-ready DevOps backend server built in Rust. Provides REST API with JWT authentication, PostgreSQL database, and Redis caching.

## Quick Start

### Prerequisites

- Rust 1.75+ (`rustup update stable`)
- Docker & Docker Compose
- (Optional) PostgreSQL 16+ and Redis 7+ if not using Docker

### Development Setup

1. **Clone and setup environment:**
   ```bash
   cd vibe-devops-server
   cp .env.example .env
   ```

2. **Start dependencies (PostgreSQL & Redis):**
   ```bash
   docker-compose up -d postgres redis
   ```

3. **Run the server:**
   ```bash
   cargo run
   ```

   The server will start at `http://localhost:8080`

4. **Check health:**
   ```bash
   curl http://localhost:8080/api/v1/health
   ```

### Running with Docker

```bash
# Build and run everything
docker-compose up --build

# Or just the dependencies
docker-compose up -d postgres redis
```

## API Endpoints

### Health
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/health` | Basic health check |
| GET | `/api/v1/health/ready` | Readiness probe (checks DB & Redis) |
| GET | `/api/v1/health/live` | Liveness probe |

### Authentication
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/auth/register` | Register new user |
| POST | `/api/v1/auth/login` | Login with email/password |
| POST | `/api/v1/auth/refresh` | Refresh access token |
| POST | `/api/v1/auth/logout` | Logout (requires auth) |

### Users
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/users/me` | Get current user |
| PUT | `/api/v1/users/me` | Update current user |
| DELETE | `/api/v1/users/me` | Delete current user |
| GET | `/api/v1/users/me/sessions` | List active sessions |

## Configuration

Configuration is done via environment variables. See `.env.example` for all options.

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_HOST` | `0.0.0.0` | Server bind address |
| `SERVER_PORT` | `8080` | Server port |
| `DATABASE_URL` | - | PostgreSQL connection URL |
| `REDIS_URL` | - | Redis connection URL |
| `JWT_SECRET` | - | Secret for JWT signing (min 32 chars) |
| `ENVIRONMENT` | `development` | Environment name |

## Development

### Project Structure

```
src/
├── main.rs           # Entry point
├── config/           # Configuration
├── db/               # Database connection & migrations
├── redis/            # Redis connection & utilities
├── models/           # Data models
├── repositories/     # Database access layer
├── services/         # Business logic
│   └── auth/         # Authentication service
├── api/              # HTTP API
│   ├── router.rs     # Route definitions
│   ├── handlers/     # Request handlers
│   ├── extractors/   # Custom extractors
│   └── responses/    # Response types
├── errors/           # Error handling
└── utils/            # Utility functions
```

### Running Tests

```bash
cargo test
```

### Database Migrations

Migrations run automatically on startup. Migration files are in `migrations/`.

## Status

This is an MVP with the following features complete:

- ✅ Project scaffolding
- ✅ Database connection & migrations
- ✅ Basic auth (register/login/logout)
- ✅ JWT access tokens with refresh
- ✅ Health check endpoints
- ✅ Docker Compose setup

### Planned (not yet implemented)

- ⬜ Admin user system
- ⬜ WebSocket server
- ⬜ Full OpenAPI docs
- ⬜ Rate limiting
- ⬜ Email verification
- ⬜ OAuth providers

## License

MIT

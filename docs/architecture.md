# rustchat Architecture

## High-Level Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      Web / SPA Client                       │
│              (Vue 3 + TypeScript + Pinia)                   │
└─────────────────────────┬───────────────────────────────────┘
                          │ REST / WebSocket
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                    rustchat API Server                      │
│                    (Axum + Tokio)                           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────────┐ │
│  │  Auth   │  │ Channels│  │  Posts  │  │   Real-time     │ │
│  │ Module  │  │ Module  │  │ Module  │  │   WebSocket Hub │ │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────────┬────────┘ │
│       │            │            │                │          │
│  ┌────▼────────────▼────────────▼────────────────▼────────┐ │
│  │                    Service Layer                       │ │
│  └────────────────────────┬───────────────────────────────┘ │
└───────────────────────────┼─────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
   ┌────▼────┐         ┌────▼────┐         ┌────▼────┐
   │Postgres │         │  Redis  │         │   S3    │
   │ (Data)  │         │ (Cache) │         │ (Files) │
   └─────────┘         └─────────┘         └─────────┘
```

---

## Backend Architecture

### API Layer (`src/api/`)

HTTP request handling using Axum:
- Route definitions
- Request validation
- Response formatting
- Middleware (auth, logging, CORS)

### Service Layer

Business logic modules:
- **auth** — Authentication, JWT tokens, password hashing
- **orgs** — Organization management
- **teams** — Team membership and permissions
- **channels** — Channel CRUD and access control
- **posts** — Message handling, threads, reactions
- **files** — Upload/download, S3 integration
- **realtime** — WebSocket connections, event fan-out

### Data Layer (`src/db/`)

PostgreSQL with SQLx:
- Connection pool management
- Compile-time checked queries
- Migration runner

### Configuration (`src/config/`)

Environment-based configuration using the `config` crate.

### Error Handling (`src/error/`)

Structured error types with HTTP status mapping.

### Telemetry (`src/telemetry/`)

Structured JSON logging with `tracing`.

---

## Frontend Architecture (Refactored 2025)

The frontend has been refactored from a flat, mixed-concern architecture to a **feature-based, layered architecture**.

### Directory Structure

```
frontend/src/
├── core/                          # Shared primitives
│   ├── entities/                  # Domain models (User, Message, Channel, etc.)
│   ├── errors/                    # Error hierarchy (AppError)
│   ├── repositories/              # Base repository interfaces
│   ├── services/                  # Shared utilities (retry logic)
│   ├── types/                     # Type utilities (Result<T,E>)
│   ├── websocket/                 # WebSocket infrastructure
│   └── index.ts                   # Public API exports
│
├── features/                      # Domain features (13 modules)
│   ├── auth/                      # Authentication & session
│   ├── calls/                     # WebRTC voice/video calls
│   ├── channels/                  # Channel management
│   ├── messages/                  # Messaging & threads
│   ├── teams/                     # Team management
│   ├── presence/                  # User presence & typing
│   ├── unreads/                   # Unread message counts
│   ├── preferences/               # User preferences
│   ├── theme/                     # UI theming
│   ├── ui/                        # UI state (modals, sidebar)
│   ├── admin/                     # Admin console
│   ├── playbooks/                 # Incident response
│   └── config/                    # Site configuration
│
├── api/                           # API clients (unchanged)
├── components/                    # Vue components
├── composables/                   # Vue composables
└── stores/                        # Legacy stores (deprecated)
```

### Feature Module Structure

Each feature follows the same layered pattern:

```
features/[feature]/
├── repositories/[feature]Repository.ts  # Data access layer
├── services/[feature]Service.ts         # Business logic
├── stores/[feature]Store.ts             # State management
├── handlers/[feature]SocketHandlers.ts  # WebSocket events
├── composables/use[Feature].ts          # Vue integration (optional)
└── index.ts                             # Public API
```

### Key Principles

1. **Feature-Based Organization**: Code grouped by domain, not by type
2. **Repository Pattern**: Data access abstraction
3. **Service Layer**: Business logic, orchestration, WebRTC
4. **Pure Stores**: State management only, no business logic
5. **Dependency Inversion**: No circular dependencies
6. **Single Responsibility**: Average 105 lines per file
7. **Explicit Error Handling**: Result types, AppError hierarchy
8. **Optimistic Updates**: UI responds immediately, syncs in background
9. **WebSocket Decoupling**: Feature-specific handlers
10. **Type Safety**: Branded types throughout

### Usage Example

```typescript
// Import from features
import { messageService, useMessageStore } from '@/features/messages'
import { callService } from '@/features/calls'
import { authService, useAuth } from '@/features/auth'

// WebSocket setup
import { registerWebSocketHandlers } from '@/core/websocket'
registerWebSocketHandlers()
```

### Refactoring Statistics

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| **Max file size** | 960 lines | 270 lines | **72% smaller** |
| **Average file size** | 238 lines | 105 lines | **56% smaller** |
| **Files** | 13 stores | 68 modules | Better organized |
| **Lines** | 3,100 | 7,174 | More maintainable |

---

## Data Flow

### HTTP Request (Backend)

```
Request → Middleware → Router → Handler → Service → Repository → Database
                                   ↓
Response ← JSON Serialization ← Result
```

### WebSocket Event (Backend)

```
Client ← WebSocket Hub ← Event Publisher ← Service ← Database Change
```

Detailed adapter boundaries and v1/v4 contract mapping:
`docs/websocket_architecture.md`.

### Frontend Data Flow

```
Component → Store → Service → Repository → API
                ↑
WebSocket → Handler → Service
```

---

## Database Schema

Key tables:
- `organizations` — Multi-tenant organizations
- `users` — User accounts (humans and bots)
- `teams` — Teams within organizations
- `channels` — Communication channels
- `posts` — Messages and threads
- `files` — File metadata

---

## Scalability

- **Stateless API servers** — Horizontal scaling behind load balancer
- **Redis pub/sub** — Cross-instance event propagation
- **Redis-backed calls state** — Shared call control-plane state (`docs/calls_deployment_modes.md`)
- **Connection pooling** — Efficient database connections
- **Async I/O** — Non-blocking operations throughout

---

## Documentation

- `docs/websocket_architecture.md` — WebSocket protocol details
- `docs/calls_deployment_modes.md` — Call service deployment
- `frontend/REFACTORING_FINAL.md` — Frontend refactoring details
- `frontend/MIGRATION_GUIDE.md` — Component migration guide
- `frontend/ARCHITECTURE_DIAGRAM.md` — Visual architecture
- `frontend/DEVELOPER_GUIDE.md` — Developer quick reference

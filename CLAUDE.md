# CLAUDE.md - AI Assistant Guide for Wazuh Process Tree Viewer

**Last Updated**: January 2026
**Version**: 3.0.0
**Language**: Rust (Edition 2021)

This document provides comprehensive guidance for AI assistants working on the Wazuh Process Tree Viewer codebase.

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture & Design Patterns](#architecture--design-patterns)
3. [Directory Structure](#directory-structure)
4. [Key Technologies](#key-technologies)
5. [Core Components](#core-components)
6. [Data Models](#data-models)
7. [API Endpoints](#api-endpoints)
8. [Development Workflow](#development-workflow)
9. [Code Conventions](#code-conventions)
10. [Security Considerations](#security-considerations)
11. [Common Tasks](#common-tasks)
12. [Testing Guidelines](#testing-guidelines)
13. [Troubleshooting](#troubleshooting)

---

## Project Overview

### Purpose
Enterprise-grade forensic visualization tool for Wazuh SIEM that analyzes and visualizes Windows process trees from Event ID 4688 logs.

### Key Capabilities
- **Authentication**: Session-based auth using Wazuh Indexer credentials
- **Process Analysis**: Queries Wazuh Indexer (OpenSearch) for process creation events
- **Visualization**: Builds interactive process tree graphs with D3.js/vis.js
- **Classification**: Dynamic, customizable process categorization system
- **Export**: JSON and CSV export capabilities
- **Real-time Filtering**: Client-side filtering without re-querying

### Performance Targets
- Startup time: < 0.5s
- Memory usage: < 50MB typical
- Support 100+ concurrent users
- Process trees with 1000+ nodes

---

## Architecture & Design Patterns

### Architectural Style
**3-Tier Architecture**:
```
┌─────────────────────────────────────────┐
│  Presentation Layer (HTML/JS)          │
│  - templates/*.html                     │
│  - static/js/*.js                       │
└─────────────────────────────────────────┘
                 ↕ HTTPS
┌─────────────────────────────────────────┐
│  Application Layer (Rust/Actix-web)    │
│  - handlers/ (HTTP handlers)            │
│  - services/ (business logic)           │
│  - middleware/ (cross-cutting)          │
└─────────────────────────────────────────┘
                 ↕
┌─────────────────────────────────────────┐
│  Data Layer                             │
│  - SQLite (sessions, classifications)   │
│  - Wazuh Indexer (process events)       │
└─────────────────────────────────────────┘
```

### Design Patterns Used

1. **Repository Pattern**: `services/database.rs` abstracts data access
2. **Service Layer**: Business logic isolated in `services/`
3. **Dependency Injection**: Via `actix-web::web::Data`
4. **Error Handling**: Custom `AppError` enum with `ResponseError` trait
5. **Middleware Pattern**: Security headers, session management
6. **Builder Pattern**: Process tree construction in `process_tree.rs`

### Module Organization
```
src/
├── main.rs           # Application entry point, server setup
├── config.rs         # Configuration from environment
├── error.rs          # Centralized error types
├── middleware.rs     # Custom middleware (SecurityHeaders)
├── utils.rs          # Utility functions
├── models/           # Data structures, DTOs
│   └── mod.rs
├── handlers/         # HTTP request handlers (controllers)
│   ├── mod.rs        # Route configuration
│   ├── auth.rs       # Login, logout, session check
│   ├── process.rs    # Process analysis, export
│   └── classification.rs  # CRUD for classifications
└── services/         # Business logic layer
    ├── mod.rs
    ├── database.rs   # SQLite operations
    ├── wazuh.rs      # Wazuh Indexer queries
    ├── classification.rs  # Classification matching
    └── process_tree.rs    # Tree building algorithm
```

---

## Directory Structure

```
wazuh-process-tree-viewer/
├── Cargo.toml                    # Rust dependencies and build config
├── README.md                     # User-facing documentation
├── CLAUDE.md                     # This file - AI assistant guide
│
├── certs/                        # SSL/TLS certificates (gitignored)
│   ├── cert.pem                  # SSL certificate
│   └── key.pem                   # Private key
│
├── src/                          # Rust source code
│   ├── main.rs                   # Entry point: server initialization
│   ├── config.rs                 # AppConfig struct, env loading
│   ├── error.rs                  # AppError enum, error handling
│   ├── middleware.rs             # SecurityHeaders middleware
│   ├── utils.rs                  # Helper functions
│   │
│   ├── models/                   # Data models and DTOs
│   │   └── mod.rs                # All model definitions
│   │
│   ├── handlers/                 # HTTP request handlers
│   │   ├── mod.rs                # Route configuration
│   │   ├── auth.rs               # Authentication endpoints
│   │   ├── process.rs            # Process analysis endpoints
│   │   └── classification.rs     # Classification CRUD endpoints
│   │
│   └── services/                 # Business logic services
│       ├── mod.rs                # Service exports
│       ├── database.rs           # SQLite database service
│       ├── wazuh.rs              # Wazuh Indexer client
│       ├── classification.rs     # Classification service
│       └── process_tree.rs       # Process tree builder
│
├── templates/                    # HTML templates
│   ├── login.html                # Login page
│   ├── dashboard.html            # Main analysis dashboard
│   └── settings.html             # Classification settings
│
├── static/                       # Static assets
│   └── js/
│       └── dashboard.js          # Dashboard JavaScript
│
├── target/                       # Build artifacts (gitignored)
│   ├── debug/                    # Debug builds
│   └── release/                  # Release builds
│
└── wptv.db                       # SQLite database (created at runtime)
```

### Important Files for AI Assistants

| File | Purpose | When to Modify |
|------|---------|----------------|
| `src/models/mod.rs` | All data structures | Adding new fields, endpoints |
| `src/handlers/mod.rs` | Route configuration | Adding new API endpoints |
| `src/services/wazuh.rs` | Wazuh API queries | Changing query logic |
| `src/services/process_tree.rs` | Tree building | Changing visualization logic |
| `src/error.rs` | Error types | Adding new error cases |
| `Cargo.toml` | Dependencies | Adding new libraries |

---

## Key Technologies

### Core Stack
| Technology | Version | Purpose |
|------------|---------|---------|
| **Rust** | 1.75+ | System programming language |
| **Actix-web** | 4.4 | Async web framework |
| **Tokio** | 1.35 | Async runtime |
| **SQLx** | 0.7 | Async SQL toolkit (SQLite) |
| **Reqwest** | 0.11 | HTTP client for Wazuh API |
| **Serde** | 1.0 | Serialization framework |

### Security
| Crate | Purpose |
|-------|---------|
| `actix-session` | Encrypted cookie sessions |
| `argon2` | Password hashing (if needed) |
| `jsonwebtoken` | JWT handling |
| `rustls` | TLS/HTTPS via reqwest |

### Frontend
| Technology | Purpose |
|------------|---------|
| Vanilla JavaScript | Dashboard interactivity |
| D3.js / vis.js | Process tree visualization |
| HTML5/CSS3 | UI (GeForce NOW style) |

### Development Tools
```bash
cargo watch      # Auto-reload during development
cargo clippy     # Linting
cargo fmt        # Code formatting
cargo test       # Unit testing
```

---

## Core Components

### 1. Authentication System (`handlers/auth.rs`)

**Flow**:
1. User submits Wazuh Indexer credentials
2. Backend validates by querying Indexer API
3. On success, creates session in SQLite
4. Returns session cookie to client
5. Subsequent requests validated via session middleware

**Key Functions**:
```rust
login(req: LoginRequest) -> LoginResponse
logout(session: Session) -> ApiResponse<()>
check_session(session: Session) -> ApiResponse<Session>
```

**Session Storage**: `database.rs:create_session()`, `get_session_by_id()`

### 2. Process Analysis (`services/wazuh.rs` + `process_tree.rs`)

**Query Flow**:
```
1. handlers/process.rs:analyze_process_tree()
   ↓
2. services/wazuh.rs:query_process_events()
   - Builds OpenSearch query
   - Filters by agent_id, time_range, event.code:4688
   ↓
3. services/process_tree.rs:build_tree()
   - Parses ProcessEvent objects
   - Builds parent-child relationships
   - Applies classifications
   - Detects orphans
   ↓
4. Returns ProcessTree (nodes, edges, orphans)
```

**OpenSearch Query Structure**:
```json
{
  "query": {
    "bool": {
      "must": [
        {"term": {"agent.id": "001"}},
        {"term": {"event.code": "4688"}},
        {"range": {"@timestamp": {"gte": "...", "lte": "..."}}}
      ]
    }
  },
  "size": 10000,
  "sort": [{"@timestamp": "asc"}]
}
```

### 3. Classification System (`services/classification.rs`)

**How It Works**:
1. Classifications stored in SQLite with patterns (JSON array)
2. When building tree, each process checked against patterns
3. Patterns matched via case-insensitive substring search
4. First match wins (sorted by priority DESC)
5. Default classification if no match

**Classification Fields**:
```rust
pub struct ProcessClassification {
    pub id: String,           // UUID
    pub name: String,         // e.g., "Web Browsers"
    pub patterns: String,     // JSON: ["chrome", "firefox"]
    pub color_bg: String,     // Hex color
    pub color_border: String, // Hex color
    pub priority: i32,        // Higher = checked first
    pub enabled: bool,        // Active/inactive
}
```

**Default Classifications**:
- System Processes (priority 100)
- Web Browsers (priority 90)
- Development Tools (priority 80)
- Office Applications (priority 70)
- Security Tools (priority 60)
- Suspicious (priority 200)

### 4. Database Service (`services/database.rs`)

**Tables**:

**sessions**:
```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    indexer_host TEXT NOT NULL,
    indexer_port INTEGER NOT NULL,
    indexer_username TEXT NOT NULL,
    indexer_password TEXT NOT NULL,  -- Encrypted
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    last_accessed TEXT NOT NULL
);
```

**classifications**:
```sql
CREATE TABLE classifications (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    patterns TEXT NOT NULL,      -- JSON array
    color_bg TEXT NOT NULL,
    color_border TEXT NOT NULL,
    icon TEXT,
    priority INTEGER NOT NULL,
    enabled BOOLEAN NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

**audit_logs**:
```sql
CREATE TABLE audit_logs (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    action TEXT NOT NULL,
    resource TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    details TEXT  -- JSON
);
```

---

## Data Models

### Key Models (`src/models/mod.rs`)

**ProcessEvent**: Raw event from Wazuh Indexer
```rust
pub struct ProcessEvent {
    pub timestamp: String,
    pub agent_id: String,
    pub process_id: String,       // PID
    pub parent_process_id: String, // PPID
    pub process_name: String,
    pub process_path: String,
    pub command_line: String,
    pub user: String,
    // ... other fields
}
```

**ProcessNode**: Node in visualization graph
```rust
pub struct ProcessNode {
    pub id: String,               // PID
    pub label: String,            // Short name
    pub title: String,            // HTML tooltip
    pub color: NodeColor,
    pub classification: String,
    // ... visualization properties
}
```

**ProcessTree**: Complete tree structure
```rust
pub struct ProcessTree {
    pub nodes: Vec<ProcessNode>,
    pub edges: Vec<ProcessEdge>,
    pub orphans: Vec<OrphanProcess>,  // Processes with missing parents
    pub total_processes: usize,
}
```

**ApiResponse**: Standard API wrapper
```rust
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}
```

---

## API Endpoints

### Authentication
| Method | Endpoint | Handler | Description |
|--------|----------|---------|-------------|
| POST | `/api/auth/login` | `auth::login` | Authenticate with Wazuh |
| POST | `/api/auth/logout` | `auth::logout` | Destroy session |
| GET | `/api/auth/check` | `auth::check_session` | Validate session |

### Process Analysis
| Method | Endpoint | Handler | Description |
|--------|----------|---------|-------------|
| POST | `/api/process/analyze` | `process::analyze_process_tree` | Build process tree |
| POST | `/api/process/export` | `process::export_process_tree` | Export as JSON/CSV |
| GET | `/api/process/agent/{id}` | `process::get_agent_info` | Get agent details |

### Classifications
| Method | Endpoint | Handler | Description |
|--------|----------|---------|-------------|
| GET | `/api/classifications` | `classification::get_all_classifications` | List all |
| POST | `/api/classifications` | `classification::create_classification` | Create new |
| GET | `/api/classifications/{id}` | `classification::get_classification` | Get by ID |
| PUT | `/api/classifications/{id}` | `classification::update_classification` | Update existing |
| DELETE | `/api/classifications/{id}` | `classification::delete_classification` | Delete |
| POST | `/api/classifications/defaults` | `classification::initialize_defaults` | Load defaults |

### Frontend Pages
| Method | Endpoint | Handler | Template |
|--------|----------|---------|----------|
| GET | `/` | `index` | `login.html` |
| GET | `/login` | `login_page` | `login.html` |
| GET | `/dashboard` | `dashboard_page` | `dashboard.html` |
| GET | `/settings` | `settings_page` | `settings.html` |

### Health Check
| Method | Endpoint | Response |
|--------|----------|----------|
| GET | `/api/health` | `{"status": "healthy", "version": "3.0.0"}` |

---

## Development Workflow

### Environment Setup

1. **Install Rust**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Create `.env` file** (optional, has defaults):
   ```env
   HOST=0.0.0.0
   PORT=5443
   DATABASE_URL=sqlite://wptv.db
   SECRET_KEY=your-secret-key-here
   LOG_LEVEL=info
   CACHE_TTL=300
   ```

3. **Generate SSL certificates**:
   ```bash
   mkdir -p certs
   openssl req -x509 -newkey rsa:4096 -nodes \
     -keyout certs/key.pem -out certs/cert.pem \
     -days 365 -subj "/CN=localhost"
   ```

### Build & Run

```bash
# Development build
cargo build

# Run with auto-reload
cargo install cargo-watch
cargo watch -x run

# Run with debug logging
RUST_LOG=debug cargo run

# Production build (optimized)
cargo build --release
./target/release/wazuh-process-tree-viewer
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Run tests
cargo test

# Check without building
cargo check
```

### Database Migrations

Migrations run automatically in `main.rs:main()` via:
```rust
db.run_migrations().await.expect("Failed to run migrations");
```

Located in: `src/services/database.rs:run_migrations()`

---

## Code Conventions

### Rust Style Guide

1. **Naming Conventions**:
   - Types (structs, enums): `PascalCase`
   - Functions, variables: `snake_case`
   - Constants: `SCREAMING_SNAKE_CASE`
   - Modules: `snake_case`

2. **Error Handling**:
   - Use `Result<T, AppError>` for fallible operations
   - Propagate errors with `?` operator
   - Convert external errors to `AppError`
   - Log errors at appropriate level (trace, debug, info, warn, error)

3. **Async Functions**:
   - All handlers must be `async fn`
   - Database operations are async (SQLx)
   - HTTP requests are async (reqwest)
   - Use `await` for async operations

4. **Documentation**:
   - Public API functions should have doc comments (`///`)
   - Complex algorithms should have inline comments
   - Module-level documentation in `mod.rs` files

### File Organization

- **One concern per file**: Don't mix authentication and process logic
- **Group related functions**: All Wazuh queries in `wazuh.rs`
- **Keep handlers thin**: Business logic in services, not handlers
- **Models in one place**: All structs in `models/mod.rs`

### Import Style
```rust
// Standard library
use std::collections::HashMap;
use std::time::Duration;

// External crates (alphabetical)
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Internal modules
use crate::error::AppError;
use crate::models::ProcessEvent;
```

### Error Handling Pattern
```rust
pub async fn some_operation() -> Result<Data, AppError> {
    let result = external_call()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !result.is_valid() {
        return Err(AppError::BadRequest("Invalid data".into()));
    }

    Ok(result.data)
}
```

---

## Security Considerations

### Critical Security Patterns

1. **Session Management**:
   - Sessions expire after 2 hours (configurable)
   - Session IDs are UUIDs (random, unguessable)
   - Credentials encrypted in database
   - Secure cookie flags set (HttpOnly, Secure)

2. **Input Validation**:
   - All user inputs validated via serde
   - Query parameters sanitized
   - HTML escaping in templates
   - SQL injection prevented by SQLx prepared statements

3. **HTTPS/TLS**:
   - All traffic over HTTPS
   - TLS enforced via rustls
   - Certificate validation for Wazuh Indexer

4. **CORS**:
   - Restricted to same origin
   - Only allowed methods: GET, POST, PUT, DELETE
   - Configured in `main.rs`

5. **Rate Limiting**:
   - Implemented via `governor` crate
   - Default: 20 requests/minute per IP
   - Configurable via environment

6. **Error Messages**:
   - Internal errors logged, not exposed to client
   - Generic error messages returned
   - Detailed errors only in debug mode

### Security Checklist for Changes

When modifying code, verify:

- [ ] User inputs are validated
- [ ] Errors don't leak sensitive information
- [ ] SQL queries use prepared statements
- [ ] Authentication required for protected endpoints
- [ ] Secrets not logged or returned in responses
- [ ] CORS settings appropriate
- [ ] Session validation present
- [ ] HTML/JS injection prevented

---

## Common Tasks

### Adding a New API Endpoint

1. **Define request/response models** in `src/models/mod.rs`:
   ```rust
   #[derive(Debug, Deserialize)]
   pub struct NewFeatureRequest {
       pub param1: String,
       pub param2: i32,
   }
   ```

2. **Create handler** in `src/handlers/` (or add to existing file):
   ```rust
   pub async fn new_feature_handler(
       req: web::Json<NewFeatureRequest>,
       db: web::Data<DatabaseService>,
   ) -> Result<HttpResponse, AppError> {
       // Business logic here
       Ok(HttpResponse::Ok().json(ApiResponse::success(data)))
   }
   ```

3. **Register route** in `src/handlers/mod.rs`:
   ```rust
   .route("/api/feature/new", web::post().to(new_feature_handler))
   ```

4. **Add tests** (if applicable):
   ```rust
   #[cfg(test)]
   mod tests {
       #[actix_web::test]
       async fn test_new_feature() {
           // Test implementation
       }
   }
   ```

### Adding a Database Table

1. **Update migration** in `src/services/database.rs:run_migrations()`:
   ```rust
   sqlx::query(
       "CREATE TABLE IF NOT EXISTS new_table (
           id TEXT PRIMARY KEY,
           field1 TEXT NOT NULL,
           created_at TEXT NOT NULL
       )"
   )
   .execute(&self.pool)
   .await?;
   ```

2. **Create model** in `src/models/mod.rs`:
   ```rust
   #[derive(Debug, Serialize, Deserialize, FromRow)]
   pub struct NewTable {
       pub id: String,
       pub field1: String,
       pub created_at: DateTime<Utc>,
   }
   ```

3. **Add service methods** in `src/services/database.rs`:
   ```rust
   pub async fn create_new_table_entry(&self, entry: NewTable) -> Result<(), AppError> {
       sqlx::query("INSERT INTO new_table (id, field1, created_at) VALUES (?, ?, ?)")
           .bind(&entry.id)
           .bind(&entry.field1)
           .bind(&entry.created_at.to_rfc3339())
           .execute(&self.pool)
           .await
           .map_err(|e| AppError::Database(e.to_string()))?;
       Ok(())
   }
   ```

### Modifying Wazuh Query Logic

File: `src/services/wazuh.rs`

1. Locate `query_process_events()` function
2. Modify OpenSearch query JSON structure
3. Update field mappings if Wazuh schema changes
4. Test with real Wazuh instance
5. Update error handling for new failure modes

### Adding a Classification

Via API (runtime):
```bash
curl -X POST https://localhost:5443/api/classifications \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Database Tools",
    "patterns": ["mysql", "postgres", "mongodb"],
    "color_bg": "#4CAF50",
    "color_border": "#2E7D32",
    "priority": 75,
    "enabled": true
  }'
```

Via code (default classifications):
Edit `src/handlers/classification.rs:initialize_defaults()`

### Updating Dependencies

```bash
# Check for outdated dependencies
cargo outdated

# Update Cargo.lock
cargo update

# Update specific dependency
cargo update -p actix-web

# After updates, rebuild and test
cargo build --release
cargo test
```

---

## Testing Guidelines

### Unit Tests

Place tests in same file as implementation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_matching() {
        let classification = ProcessClassification {
            patterns: r#"["chrome", "firefox"]"#.to_string(),
            // ... other fields
        };

        let patterns = classification.get_patterns();
        assert_eq!(patterns, vec!["chrome", "firefox"]);
    }
}
```

### Integration Tests

Create `tests/` directory for integration tests:

```rust
// tests/integration_test.rs
use wazuh_process_tree_viewer::*;

#[actix_web::test]
async fn test_health_endpoint() {
    // Test setup
    let app = test::init_service(App::new().configure(configure_routes)).await;
    let req = test::TestRequest::get().uri("/api/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}
```

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_classification_matching

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test integration_test
```

### Manual Testing Workflow

1. **Start server**: `cargo run`
2. **Test login**: Open `https://localhost:5443/login`
3. **Check logs**: Verify no errors in console
4. **Test API**: Use curl or Postman
5. **Check database**: `sqlite3 wptv.db "SELECT * FROM sessions;"`

---

## Troubleshooting

### Common Build Errors

**Error**: `error: linker 'cc' not found`
```bash
# Ubuntu/Debian
sudo apt install build-essential

# CentOS/RHEL
sudo yum install gcc
```

**Error**: `failed to run custom build command for openssl-sys`
```bash
# Ubuntu/Debian
sudo apt install libssl-dev pkg-config

# macOS
brew install openssl
```

**Error**: `could not find system library 'sqlite3'`
```bash
# Ubuntu/Debian
sudo apt install libsqlite3-dev

# Already bundled in SQLx, shouldn't happen
```

### Runtime Issues

**Database locked**:
- Close any open SQLite connections
- Delete `wptv.db-wal` and `wptv.db-shm` files
- Restart application

**Port already in use**:
```bash
# Find process using port 5443
lsof -i :5443
# Kill process
kill -9 <PID>
```

**Cannot connect to Wazuh Indexer**:
- Verify Indexer is running: `curl -k https://indexer:9200`
- Check firewall allows port 9200
- Verify credentials are correct
- Check TLS certificate validation

**Session expired immediately**:
- Check system clock is synchronized
- Verify `SECRET_KEY` is set correctly
- Check session table in database

### Debugging Tips

1. **Enable debug logging**:
   ```bash
   RUST_LOG=debug cargo run
   ```

2. **Check database contents**:
   ```bash
   sqlite3 wptv.db
   .tables
   SELECT * FROM sessions;
   .quit
   ```

3. **Test Wazuh query manually**:
   ```bash
   curl -k -u user:pass https://indexer:9200/wazuh-alerts-*/_search \
     -H 'Content-Type: application/json' \
     -d '{"query":{"term":{"agent.id":"001"}},"size":1}'
   ```

4. **Inspect HTTP traffic**:
   - Use browser DevTools Network tab
   - Check request/response headers
   - Verify JSON payloads

5. **Memory profiling**:
   ```bash
   cargo install cargo-instruments
   cargo instruments --release --template time
   ```

---

## Glossary

- **WPTV**: Wazuh Process Tree Viewer
- **Event 4688**: Windows process creation event
- **Wazuh Indexer**: OpenSearch-based log storage
- **Classification**: Category assigned to process based on name patterns
- **Orphan Process**: Process whose parent is not in the tree
- **Session**: Authenticated user session with encrypted credentials
- **Process Tree**: Hierarchical graph of parent-child process relationships

---

## Additional Resources

### External Documentation
- [Actix-web Documentation](https://actix.rs/docs/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Tokio Documentation](https://tokio.rs/tokio/tutorial)
- [Wazuh API Reference](https://documentation.wazuh.com/current/user-manual/api/reference.html)
- [OpenSearch Query DSL](https://opensearch.org/docs/latest/query-dsl/)

### Rust Learning
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Async Book](https://rust-lang.github.io/async-book/)

### Project-Specific
- `README.md`: User-facing documentation
- `Cargo.toml`: Dependency versions and metadata
- Inline code comments: Implementation details

---

## Changelog (CLAUDE.md)

### 2026-01-14
- Initial creation of CLAUDE.md
- Documented all major systems and patterns
- Added comprehensive troubleshooting guide
- Included common tasks for AI assistants

---

## Questions for AI Assistants to Ask

When uncertain about a task, consider asking the user:

1. **Scope**: "Should this change affect only [specific component] or be system-wide?"
2. **Breaking changes**: "This change might affect [X]. Is backward compatibility required?"
3. **Security**: "This involves user data. Should I add audit logging?"
4. **Performance**: "This could impact performance with large datasets. Should I add caching?"
5. **Testing**: "Should I add tests for this functionality?"
6. **Configuration**: "Should this be configurable via environment variables?"

---

**End of CLAUDE.md**

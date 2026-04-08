# RustMUD - A Rust MUD Engine

## Overview

RustMUD is a 1:1 Rust port of the txpike9 MUD engine. This project aims to rewrite txpike9 with Rust's performance and safety while maintaining the same architecture and design patterns for easier maintenance.

## Architecture

```
rustmud/
├── src/
│   ├── main.rs              # Main entry point
│   ├── core/                # Core type system
│   │   ├── mod.rs           # Core modules (ObjectId, Frame, Backtrace)
│   │   ├── object.rs        # Object system (Pike: object)
│   │   ├── mapping.rs       # Mapping type (Pike: mapping)
│   │   ├── array.rs         # Array type (Pike: array)
│   │   ├── value.rs         # Value type (Pike: mixed)
│   │   ├── error.rs         # Error handling (Pike: handle_error)
│   │   └── program.rs       # Program loader (Pike: program)
│   ├── pikenv/              # Pike environment (txpike9/pikenv/)
│   │   ├── mod.rs
│   │   ├── pikenv.rs        # Main server entry (Pike: pikenv.pike)
│   │   ├── conn.rs          # Connection handler (Pike: conn.pike)
│   │   ├── connd.rs         # Connection manager (Pike: connd.pike)
│   │   ├── efuns.rs         # Built-in functions (Pike: efuns.pike)
│   │   ├── config.rs        # Configuration system
│   │   ├── pike_save.rs     # Pike save_object format parser
│   │   └── gc_manager.rs    # GC Manager
│   ├── gamenv/              # Game environment (txpike9/gamenv/)
│   │   ├── mod.rs
│   │   ├── master.rs        # Master controller (Pike: master.pike)
│   │   ├── user.rs          # User object (Pike: clone/user)
│   │   ├── cmds.rs          # Command system
│   │   ├── daemons.rs       # Daemon system
│   │   ├── inherit.rs       # Inherit base classes
│   │   ├── d.rs             # Rooms/World
│   │   ├── clone.rs         # Cloneable objects
│   │   ├── data.rs          # Data definitions
│   │   └── http_api/        # HTTP API module
│   │       ├── mod.rs       # HTTP API main
│   │       ├── auth.rs      # TXD authentication
│   │       ├── virtual_conn.rs # Virtual connection pool
│   │       ├── command_queue.rs # Command queue
│   │       ├── handlers.rs  # HTTP handlers
│   │       └── utils.rs     # Utility functions
│   └── web/                 # Web frontend
│       ├── static/          # Static files
│       └── templates/       # HTML templates
├── Dockerfile               # Docker image definition
├── docker-compose.yml       # Docker Compose configuration
└── .dockerignore           # Docker ignore patterns
```

## Mapping Table

| txpike9 (Pike) | rustmud (Rust) | Description |
|----------------|----------------|-------------|
| pikenv.pike | pikenv/pikenv.rs | Main entry point |
| conn.pike | pikenv/conn.rs | Connection handler |
| efuns.pike | pikenv/efuns.rs | Built-in functions |
| connd.pike | pikenv/connd.rs | Connection daemon |
| master.pike | gamenv/master.rs | Master controller |
| object | core/object.rs | Object system |
| mapping | core/mapping.rs | Key-value mapping |
| save_object() | serde Serialize | Object serialization |
| restore_object() | serde Deserialize | Object deserialization |
| http_api/ | gamenv/http_api/ | HTTP/WebSocket API |

## Feature Comparison

| Feature | Pike Version | Rust Version | Advantage |
|---------|--------------|--------------|-----------|
| Performance | Interpreted | Compiled (AOT) | Near C/C++ speed |
| Concurrency | Single-threaded + coroutines | Multi-threaded + async (Tokio) | True multi-core utilization |
| Memory | Ref counting + GC | Compile-time safety | No GC pauses |
| Types | Dynamic typing | Static typing | Compile-time checks |
| Deployment | Pike runtime | Single binary | Simplified deployment |

## txpike9 Compatibility

RustMUD is designed to be fully compatible with existing txpike9 deployments:

### User Data Migration

RustMUD can read and write txpike9 user data files directly:

```rust
// User files are stored in txpike9 format
// Path: gamenv/u/XX/XXXXXX.o
// Example: gamenv/u/00/tx0100.o

// Automatic migration on first load
let mut user = User::new("tx0100".to_string());
user.load()?;  // Automatically loads from txpike9 format

// Data is saved in both formats
user.save()?;  // Saves as both JSON (RustMUD) and Pike format (txpike9)
```

### Pike Save Object Format

RustMUD includes a complete parser for the Pike save_object format:

```
#~/gamenv/clone/user.pike
name "tx0100"
name_newbei "tx0100"
level 10
exp 100000
hp 100
hp_max 100
qi 50
qi_max 50
shen 50
shen_max 50
potential 100
money 0
password "hashed_password"
login_time 1691814226
online_time 723
```

### Frontend Compatibility

The existing Vue frontend from txpike9 connects without modification:
- Same WebSocket protocol
- Same TXD authentication
- Same command interface
- Same color code support (§ codes)

## Quick Start

### Local Development

```bash
# Clone the repository
git clone https://github.com/your-org/rustmud.git
cd rustmud

# Build
cargo build --release

# Run
cargo run --release

# Set environment variables
export GAME_AREA=tx01
export ROOT=/path/to/mudlib
export PORT=9999
```

### Docker Deployment

```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f rustmud

# Stop services
docker-compose down

# Rebuild after changes
docker-compose up -d --build
```

### Docker Build Options

```bash
# Build only the game server
docker build -t rustmud:latest .

# Build with custom txpike9 data mount
docker build -t rustmud:custom .
docker run -v /path/to/txpike9:/usr/local/games/txpike9:ro rustmud:custom
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| GAME_AREA | tx01 | Game area ID |
| ROOT | current directory | Mudlib root directory |
| PORT | 9999 | MUD listening port |
| IP | 0.0.0.0 | Listening IP address |
| LOG_PREFIX | 9999 | Log file prefix |
| HTTP_PORT | 8080 | HTTP/WebSocket port |
| DATABASE_URL | | MySQL connection string |
| RUST_LOG | info | Logging level |

## HTTP API

RustMUD provides a RESTful HTTP/WebSocket API compatible with the existing Vue frontend:

### WebSocket Endpoint
```
ws://localhost:8080/ws
```

### REST API Endpoints
```
POST /api/command    - Execute a command
GET  /api/status     - Server status
GET  /api/user/:id   - User information
GET  /api/room/:id   - Room information
```

### Authentication
Uses TXD token format compatible with txpike9:
```
Authorization: TXD <encoded_token>
```

## Dependencies

- **tokio** - Async runtime
- **serde** - Serialization/deserialization
- **sqlx** - Database (MySQL)
- **axum** - HTTP/WebSocket server
- **bincode** - Binary serialization
- **tracing** - Logging and instrumentation

## Development Roadmap

- [x] Core type system (ObjectId, Value, Mapping, Array)
- [x] Connection handling (TCP, WebSocket)
- [x] HTTP API with TXD authentication
- [x] Virtual connection pool
- [x] Command queue system
- [x] Pike save_object format parser
- [ ] Complete command system implementation
- [ ] Full Daemon system compatibility
- [ ] Room/NPC/Item systems
- [ ] Combat system
- [ ] Skill system
- [ ] Guild/Bang system
- [ ] Autofight system

## License

MIT License

## References

- Original project: [txpike9](https://github.com/your-org/txpike9)

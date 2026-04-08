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

## Quick Start

```bash
# Build
cargo build --release

# Run
cargo run --release

# Set environment variables
export GAME_AREA=tx01
export ROOT=/path/to/mudlib
export PORT=9999
export HTTP_PORT=8080
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

## Dependencies

- **tokio** - Async runtime
- **serde** - Serialization/deserialization
- **sqlx** - Database (MySQL)
- **axum** - HTTP/WebSocket server

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

## Frontend Integration

The existing Vue frontend from txpike9 can connect directly to RustMUD:

1. **WebSocket Connection** - Real-time bidirectional communication
2. **Command Execution** - Same command interface
3. **TXD Authentication** - Compatible authentication system
4. **Color Codes** - Support for § color codes

## Development Status

Current phase: Initial development, core framework complete.

## License

MIT License

## References

- Original project: [txpike9](https://github.com/your-org/txpike9)

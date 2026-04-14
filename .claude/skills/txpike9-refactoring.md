# txpike9 Refactoring Skill

## Description

This skill contains knowledge for refactoring Rust MUD codebases to follow txpike9's dynamic architecture patterns. Use this when the user asks to refactor MUD code, improve modularity, or make the codebase more flexible like txpike9.

## Core Concepts

### txpike9 Architecture Philosophy

txpike9 uses a highly modular, dynamic architecture that enables:
- **Runtime composition** over static inheritance
- **File-based command discovery** (787 command files)
- **Daemon singletons** for global services
- **Loose coupling** between components
- **Easy extensibility** without changing existing code

### Key Architectural Patterns

#### 1. Command Plugin System

Instead of hardcoded command dispatch, use a dynamic registry:

```rust
// Command trait with async handler
#[async_trait]
pub trait CommandHandler: Send + Sync {
    async fn handle(&self, ctx: CommandContext) -> CommandResult;
    fn metadata(&self) -> &CommandMetadata;
}

// Registry for dynamic dispatch
pub struct CommandRegistry {
    commands: HashMap<String, Arc<dyn CommandHandler>>,
    aliases: HashMap<String, String>,
    categories: HashMap<CommandCategory, Vec<String>>,
}
```

**Benefits:**
- Add new commands without modifying core dispatch logic
- Support command aliases dynamically
- Categorize commands for help system
- Test commands in isolation

#### 2. Feature Composition System

Instead of static traits, use runtime feature composition:

```rust
pub trait Feature: Any + Send + Sync + Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_box(&self) -> Box<dyn Feature>;
}

pub struct FeatureContainer {
    features: HashMap<TypeId, Box<dyn Feature>>,
}

// Concrete features
pub struct FightFeature { pub hp: i32, pub attack: i32, ... }
pub struct InventoryFeature { pub items: Vec<String>, ... }
pub struct DialogFeature { pub topics: Vec<String>, ... }
```

**Benefits:**
- Add/remove features at runtime
- Type-safe downcasting with Any
- Compose entities from multiple features
- Similar to txpike9's inherit mechanism

#### 3. Daemon System

Unified lifecycle management for global services:

```rust
#[async_trait]
pub trait Daemon: Send + Sync + Any + Debug {
    async fn initialize(&mut self) -> Result<(), DaemonError>;
    async fn start(&mut self) -> Result<(), DaemonError>;
    async fn stop(&mut self) -> Result<(), DaemonError>;
    fn state(&self) -> DaemonState;
    async fn heartbeat(&mut self) -> bool;
}

pub struct DaemonManager {
    daemons: RwLock<HashMap<String, DaemonWrapper>>,
}
```

**Benefits:**
- Consistent lifecycle across all daemons
- Centralized health monitoring
- Graceful shutdown support
- Easy to add new daemons

## Migration Patterns

### Migrating Static Commands to Plugin System

**Before:**
```rust
// Hardcoded match statement
match cmd.as_str() {
    "look" => handle_look(ctx).await,
    "inventory" => handle_inventory(ctx).await,
    _ => unknown_command(),
}
```

**After:**
```rust
// Dynamic registration
registry.register(look::get_command());
registry.register(inventory::get_command());

// Dynamic dispatch
registry.dispatch(cmd_str, ctx).await
```

### Migrating Inheritance to Feature Composition

**Before (Static Traits):**
```rust
pub trait Npc {
    fn attack(&self) -> i32;
    fn talk(&self) -> &str;
}
```

**After (Dynamic Features):**
```rust
let mut entity = GameEntity::new(id, EntityType::Npc);
entity.add_feature(FightFeature::new(hp, attack, defense));
entity.add_feature(DialogFeature::new(dialog_id));

if let Some(fight) = entity.get_feature::<FightFeature>() {
    // Use fight capabilities
}
```

### Migrating Singletons to Daemon System

**Before:**
```rust
pub static USERD: OnceLock<RwLock<UserDaemon>> = OnceLock::new();

pub fn get_userd() -> &RwLock<UserDaemon> {
    USERD.get_or_init(|| ...)
}
```

**After:**
```rust
#[async_trait]
impl Daemon for UserDaemon {
    async fn initialize(&mut self) -> Result<(), DaemonError> { ... }
    async fn start(&mut self) -> Result<(), DaemonError> { ... }
    async fn stop(&mut self) -> Result<(), DaemonError> { ... }
}

// Register with daemon manager
let manager = get_daemon_manager();
manager.register(Box::new(UserDaemon::new())).await;
manager.initialize_all().await;
```

## Common Pitfalls

1. **RwLock Lifetime Issues**: When using RwLock with async code, create helper functions that handle the guard internally rather than returning references to guarded data.

2. **Static Initialization**: Use `once_cell::sync::Lazy` for complex static initialization instead of `OnceLock`.

3. **Downcasting**: Always use `TypeId` for type identification in trait object collections.

4. **Error Types**: Be consistent with error types - use either `std::result::Result<T, E>` or custom Result types, not both in the same module.

## File Structure

```
src/gamenv/
├── cmds/
│   ├── mod.rs           # Command module exports
│   ├── registry.rs      # Command registry
│   ├── look.rs          # Individual commands
│   └── ...
├── traits/
│   ├── mod.rs           # Trait module exports
│   ├── composition.rs   # Feature composition system
│   ├── entity.rs        # Entity wrappers
│   ├── daemon.rs        # Daemon system
│   └── ...
├── single/
│   └── daemons/
│       ├── mod.rs
│       ├── userd.rs     # Original daemons
│       ├── userd_v2.rs  # Refactored daemons
│       └── ...
└── core/
    └── command.rs       # Command trait definitions
```

## When to Use This Skill

Use this skill when:
- User asks to refactor MUD code to be more modular
- User mentions txpike9 or dynamic architecture
- User wants to add extensibility without breaking existing code
- User asks about command plugin systems
- User wants to understand how to implement runtime composition

## Testing Patterns

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_feature_container() {
        let mut container = FeatureContainer::new();
        container.add(FightFeature::new(100, 100, 10, 5));
        assert!(container.has::<FightFeature>());
    }

    #[tokio::test]
    async fn test_daemon_lifecycle() {
        let mut daemon = TestDaemon::new();
        assert!(daemon.initialize().await.is_ok());
        assert_eq!(daemon.state(), DaemonState::Running);
    }
}
```

## Example Commands

- "Refactor this command to use the plugin system"
- "Add a new feature to the entity system"
- "Create a daemon for managing X"
- "Make this more like txpike9's architecture"
- "How do I add a new command without modifying core files?"

# Rust MUD Game Development

This skill covers the Rust-based MUD (Multi-User Dungeon) game server development, including the battle system, HTTP API, hidden command system, and frontend integration.

## Project Architecture

### Core Components
- **Server Framework**: Axum (async Rust web framework)
- **Frontend**: Vue.js 3 with JSON API communication
- **Game World**: Room-based navigation with NPCs and monsters
- **Battle System**: Turn-based PK (Player Kill) daemon with skills
- **Data**: JSON-based world data migrated from Pike language

### Key Directories
- `src/gamenv/` - Core game logic
  - `http_api/` - HTTP API handlers and MUD output parsing
  - `single/daemons/` - Game daemons (PKD, spawn, etc.)
  - `world/` - Game world (rooms, NPCs, items)
  - `player_state.rs` - Player state management
  - `hidden_cmd.rs` - Hidden command system
- `data/world/` - Game world data (rooms, NPCs)
- `web/web_vue/` - Vue.js frontend
- `scripts/` - Utility scripts (Pike to JSON converter)

## Common Development Tasks

### 1. Adding a New Command

**Backend (Rust):**
Add command handler in `src/gamenv/http_api/mod.rs` in the `execute_command_internal` function:

```rust
"mycommand" => {
    if args.is_empty() {
        "Usage: mycommand <argument>".to_string()
    } else {
        let arg = args.join(" ");
        // Process command
        format!("Result: {}", arg)
    }
}
```

**With hidden command support:**
```rust
let cmd_mycommand = hidden_cmd::hide_command(userid, "mycommand").await;
// Include in actions/response
```

### 2. Fixing RwLock Deadlocks

**Pattern to avoid:**
```rust
// DON'T: Acquire same lock twice
let lock1 = self.map.write().await;
let lock2 = self.map.write().await; // DEADLOCK!
```

**Correct pattern:**
```rust
// DO: Get IDs with read lock, then acquire write lock
let id = {
    let read_lock = self.map.read().await;
    read_lock.get(key)?.clone()
};
let mut write_lock = self.map.write().await;
// Use write_lock
```

**Example from PKD:**
```rust
pub async fn select_skill(&self, player_id: &str, skill_id: &str) -> Result<String, String> {
    // Get battle_id with read lock only
    let battle_id = {
        let player_battles = self.player_battles.read().await;
        player_battles.get(player_id)?.clone()
    };
    // Then acquire write lock
    let mut battles = self.battles.write().await;
    // ...
}
```

### 3. Battle System Integration

**Starting a battle:**
```rust
use crate::gamenv::single::daemons::pkd::{PKD, CombatStats};

let challenger_stats = CombatStats { /* ... */ };
let defender_stats = CombatStats { /* ... */ };

match PKD.challenge(challenger_stats, defender_stats).await {
    Ok(battle) => battle.generate_status_for_player(userid),
    Err(e) => format!("§c{}§N", e),
}
```

**Continuing battle:**
```rust
let continue_result = pk_continue_command(userid).await;
```

**Ending battle:**
```rust
PKD.end_battle(&battle_id).await;
// This removes from battles map AND player_battles mapping
```

### 4. MUD Output Format

**WAPMUD menu format (buttons):**
```
[label:command]
```
Example: `[查看:look]` creates a button with label "查看" that sends "look" command.

**Color codes:**
- `§c` - error red
- `§Y` - yellow
- `§N` - reset
- `§H` - gold/highlight
- See `src/gamenv/http_api/utils.rs` for full color mapping

**Output structure:**
```rust
pub struct MudLine {
    pub r#type: String,  // "line" | "empty"
    pub segments: Vec<MudSegment>,
}

pub struct MudSegment {
    pub r#type: SegmentType,  // Text, Button, Input, etc.
    pub text: Option<String>,
    pub label: Option<String>,  // For buttons
    pub cmd: Option<String>,    // For buttons
    pub parts: Option<Vec<TextPart>>,  // For colored text
}
```

### 5. Debugging Battle Issues

**Add debug logging:**
```rust
println!("[DEBUG] Function called with: {:?}", param);
```

**Check battle logs:**
```bash
tail -100 /tmp/rustenv.log | grep -E "CAST|PK_CONTINUE|PKD|Battle"
```

**Common issues:**
1. **Deadlock** - Check for double lock acquisition
2. **Battle not ending** - Check if `end_battle` is called
3. **Player stuck in battle** - Check `player_battles` mapping cleanup
4. **Skill cast no response** - Check select_skill for deadlock

### 6. Frontend-Backend Communication

**API endpoint:**
```
GET /api/json?txd=<token>&cmd=<command>
```

**Response format:**
```json
{
  "status": "success",
  "lines": [/* MudLine array */],
  "room_info": { /* room data */ },
  "player_stats": { /* player data */ },
  "state": {
    "navigation": {
      "exits": [
        {"direction": "north", "label": "北：北郊", "command": "<hidden>"}
      ]
    }
  }
}
```

### 7. Data Migration (Pike to JSON)

**Run converter:**
```bash
python3 scripts/pike_to_json.py
```

**Output:** `data/world/rooms_data.json`

## File Locations Reference

| Purpose | File |
|---------|------|
| Command handlers | `src/gamenv/http_api/mod.rs` |
| Battle system | `src/gamenv/single/daemons/pkd.rs` |
| MUD output parsing | `src/gamenv/http_api/mud_output.rs` |
| Color codes | `src/gamenv/http_api/utils.rs` |
| Hidden commands | `src/gamenv/hidden_cmd.rs` |
| Player state | `src/gamenv/player_state.rs` |
| World data | `data/world/rooms_data.json` |
| Frontend | `web/web_vue/js/app.js` |
| Server logs | `/tmp/rustenv.log` |

## Build and Deploy

```bash
# Build
cargo build

# Run server
./target/debug/rustenv > /tmp/rustenv.log 2>&1 &

# Restart server
pkill -f rustenv && ./target/debug/rustenv > /tmp/rustenv.log 2>&1 &

# Check logs
tail -f /tmp/rustenv.log
```

## Important Notes

1. **Always check for deadlocks** when modifying code with RwLock
2. **Battle cleanup must remove from both maps**: `battles` and `player_battles`
3. **Use player_battles for battle existence checks**, not the battles map
4. **Test skill casting** after any battle system changes
5. **MUD output must use proper format** for frontend button rendering

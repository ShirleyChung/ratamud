# RataMUD - Copilot Instructions

RataMUD is a Rust-based MUD (Multi-User Dungeon) text adventure game engine built with Ratatui, designed to work both as a standalone terminal application and as a library (FFI) for integration with other platforms.

## Build and Test Commands

### Build
```bash
# Standard build
cargo build

# Release build
cargo build --release

# Build as library (for FFI)
cargo build --release --lib
./build_and_test.sh  # Includes library + C++ test program
```

### Run
```bash
# Run the game
cargo run

# Run specific binary
cargo run --bin main
```

### Test
```bash
# Run all tests
cargo test --lib

# Run individual test
cargo test --lib test_name

# Note: One test (test_context_dialogue) currently fails - this is a known issue
```

### Lint
```bash
# Run clippy - REQUIRED before any commit
cargo clippy

# Auto-fix clippy issues
cargo clippy --fix --allow-dirty --allow-staged
```

### Shell Test Scripts
```bash
./check_game.sh              # Validate world data and player state
./test_combat_loop.sh        # Combat system test instructions
./test_command_history.sh    # Command history functionality test
./test_item_system.sh        # Item system automated test
```

## Architecture Overview

### Threading Model: Event-Driven Single Writer

**Critical**: RataMUD uses an event-driven architecture with GameWorld as the single mutable state owner.

```
NPC Thread ─┐
Timer Thread ─┼──► Event Channel ───► GameWorld (main loop)
Input Thread ─┘                         │
                                        ├─ Updates world state
                                        ├─ Generates Messages
                                        └─ Creates RenderState ─► OutputManager ─► Terminal
```

**Key Principles** (from `Docs/EVENT_DRIVEN_RULE.md`):
1. **GameWorld is Single Writer**: Only the main thread modifies GameWorld (never Arc<Mutex<GameWorld>>)
2. **Threads produce Events/Actions**: NPC/Timer/Input threads communicate only via channels
3. **No Direct Output from NPCs**: NPCs decide intent, OutputManager handles rendering
4. **Snapshot Rendering**: RenderState is immutable data, never holds GameWorld reference

### Module Structure

```
src/
├── lib.rs              # Public module exports
├── main.rs             # Terminal UI entry point
├── app.rs              # Main game loop, event processing
├── world.rs            # World state, map management
├── map.rs              # 100x100 tile-based maps
├── person.rs           # Player & NPC entity system
├── npc_manager.rs      # NPC lifecycle management
├── npc_ai.rs           # NPC AI logic (runs in thread)
├── item.rs             # Item definitions
├── item_registry.rs    # Centralized item database
├── quest.rs            # Quest system
├── trade.rs            # Trading mechanics
│
├── event.rs            # Core event definitions
├── event_loader.rs     # Load events from JSON
├── event_executor.rs   # Execute event actions
├── event_scheduler.rs  # Cron-based event scheduling
├── game_event.rs       # GameEvent enum (cross-thread)
├── npc_view.rs         # Immutable world snapshot for NPCs
├── npc_action.rs       # NPC action intents
├── message.rs          # Output message types
│
├── time_updatable.rs   # TimeUpdatable trait
├── time_thread.rs      # Background time system
├── input.rs            # Command parsing
├── output.rs           # Output management with callbacks
├── settings.rs         # Game settings
├── ui.rs               # Ratatui UI rendering
│
└── ffi.rs              # C ABI for library mode
    ratamud.h           # C header file
```

### Data Storage

All game data persists to `worlds/{world_name}/`:
```
worlds/
└── beginWorld/
    ├── world.json          # World metadata, map list
    ├── time.json           # Game time state
    ├── maps/*.json         # Map data (100x100 grids)
    └── persons/*.json      # Player & NPC data
```

### FFI/Library Mode

RataMUD can be compiled as a library for iOS/Android/other platforms:

- **Output Callback System**: All output goes through registered callbacks (`OutputManager::trigger_callback`)
- **4 Output Types**: `MAIN` (game messages), `LOG` (system logs), `STATUS` (status bar), `SIDE` (sidebar info)
- **Thread-Safe**: Uses `Mutex` for global callback registration
- **Testing**: `test_callback.cpp` demonstrates C++ integration

See `LIB_MODE_GUIDE.md` and `Docs/C_ABI_README.md` for details.

## Key Conventions

### Code Style (CODE_RULES.md)

These rules are **mandatory** for all developers and AI assistants:

1. **Document public APIs and complex logic**: Use `///` for public functions
2. **Keep functions under 100 lines**: Extract helper functions if longer
3. **DRY principle**: Extract duplicate logic (2+ occurrences)
4. **Avoid unnecessary `.clone()`**: Use references unless crossing threads or avoiding borrow conflicts
5. **Zero warnings policy**: `cargo build` and `cargo clippy` must produce no warnings
6. **Run clippy before commit**: `cargo clippy` is required

### NPC System

- **5 NPC Types**: Merchant, Passerby, Doctor, Worker, Farmer
- **Dialogue System**: NPCs can have multiple dialogues per topic with conditional logic
- **SDL Syntax**: Three ways to set dialogues (simple, `add when`, `set when say`)
- **Relationship System**: NPCs track relationship with player (-100 to +100)
- **AI Thread**: NPCs make decisions based on `NpcView` (immutable snapshot)

### Dialogue System

NPCs support sophisticated conditional dialogue:

```bash
# Simple
sdl <NPC> <topic> <text>

# Conditional with add
sdl <NPC> <topic> add <text> when <condition>

# Set syntax (recommended)
sdl <NPC> set <topic> when <condition> say <text>
```

Conditions can check: `rel >= 50`, `hunger < 20`, `gender = male`, etc.

See `README_DIALOGUE.md` and `SDL_SYNTAX.md` for full syntax.

### Event System

- **Event-Driven**: All game logic triggered by events in main loop
- **3 Event Sources**: NPC actions, Timer ticks, User input
- **Event Scheduler**: Cron-based scheduling for timed events
- **Probability Events**: Random events with configurable probability
- **Weather System**: Example of time-based environmental events

### Map System

- **Map Size**: 100x100 tile grid
- **5 Terrain Types**: Normal, Forest, Cave, Desert, Mountain
- **Tile Properties**: `walkable`, `description`, `objects` (entities on tile)
- **Coordinates**: Player moves via `l/r/u/d` commands or `flyto x,y`

### Combat System

- **Turn-Based**: Player action triggers combat round
- **Auto-Combat Rounds**: NPCs automatically act during player's combat turn
- **Skill System**: Various skills (punch, kick, etc.) with cooldowns
- **Experience**: Awarded on combat escape/victory

### Item System

- **24 Item Types**: 6 categories (Miscellaneous, Food, Weapon, Armor, Consumable, Tool)
- **Item Registry**: Centralized item definitions with randomized generation
- **Commands**: `get`, `drop`, `use`, `give`

### Output Management

**Critical**: Never use `println!` or `eprintln!` except in `event_loader.rs` errors.

All output must go through `OutputManager`:
- `output.print(msg)` → MAIN output
- `output.log(msg)` → LOG output
- `output.set_status(msg)` → STATUS bar
- `output.set_side_content(msg)` → SIDE panel

This ensures FFI callbacks work correctly.

### Time System

- **TimeUpdatable Trait**: Entities respond to time passage
- **Game Time**: Tracks hour, minute, day
- **Time Thread**: Background thread advances time
- **Persistence**: Time state saved to `time.json`

## Important Documentation

Key reference files in repository root and `Docs/`:

- **CODE_RULES.md**: Mandatory coding standards (read first!)
- **LIB_MODE_GUIDE.md**: Library/FFI integration guide
- **REFACTOR_PLAN.md**: Multi-threading architecture plan
- **Docs/EVENT_DRIVEN_RULE.md**: Critical architecture rules
- **Docs/NPC_TALK_QUICKSTART.md**: NPC dialogue system quickstart
- **README_DIALOGUE.md**: Detailed dialogue system documentation
- **SDL_SYNTAX.md**: Dialogue condition syntax reference

## Naming Conventions

- **World Names**: English (e.g., `beginWorld`)
- **Map Names**: English (e.g., `beginMap`, `forest`, `mountain`)
- **UI Text**: Chinese (for user-facing messages)
- **Code**: English comments preferred, Chinese comments acceptable
- **Variables**: snake_case (Rust standard)
- **Types**: PascalCase (Rust standard)

## Testing Notes

- Some tests may fail (e.g., `test_context_dialogue`) - this is expected
- Manual testing often required for combat, NPC interaction, UI features
- Test scripts in repository root provide testing workflows
- Always verify no new warnings after changes

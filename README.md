# bevy_enhanced_input_extras

Shared keybinding utilities for [`bevy_enhanced_input`](https://github.com/projectharmonia/bevy_enhanced_input).

## Features

- **`action!`** — generates `InputAction` structs with minimal boilerplate
- **`event!`** — generates `Event` structs (unit or with payload) that are BRP-triggerable via `Reflect`
- **`bind_action_system!`** — wires an input action to a system through an intermediate event, so the same command can be invoked by keybinding or programmatically (e.g. via BRP `world.trigger_event`)
- **`Keybindings<C>`** — manages modifier keys (Cmd/Ctrl, Shift, Alt) with cross-platform defaults and provides helpers for spawning actions with automatic `BlockBy` rules

## Compatibility

| bevy | bevy_enhanced_input | bevy_enhanced_input_extras |
|------|---------------------|----------------------------|
| 0.18 | 0.24                | 0.1                        |

## Usage

```rust
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_enhanced_input_extras::*;

action!(PauseToggle);
event!(PauseEvent);

fn pause_command(/* ... */) {
    // ...
}

// In your app setup:
// bind_action_system!(app, PauseToggle, PauseEvent, pause_command);
```

## License

MIT OR Apache-2.0

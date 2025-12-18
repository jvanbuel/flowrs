# Focus Tracking Design

## Overview

Implement focus change tracking using crossterm's focus events. When the TUI loses focus, automatic API refreshes (Tick events) are paused to avoid unnecessary network calls while the user isn't watching.

## Behavior

- When focused: Normal operation, Tick events trigger API refreshes
- When unfocused: Tick events are discarded, no automatic refreshes
- On focus regain: Clean slate, refreshes resume normally (no queued messages to process)

## Changes

### 1. Terminal Setup (`src/main.rs`)

Enable focus change reporting on startup, disable on shutdown:

```rust
// On startup (after entering raw mode):
crossterm::terminal::enable_focus_change_reporting()?;

// On shutdown (before leaving raw mode):
crossterm::terminal::disable_focus_change_reporting()?;
```

The disable call must be in the cleanup path that runs even on panic/error.

### 2. Event Types (`src/app/events/custom.rs`)

Add two new variants to `FlowrsEvent`:

```rust
pub enum FlowrsEvent {
    Tick,
    Key(KeyEvent),
    Mouse,
    FocusGained,  // new
    FocusLost,    // new
}
```

Update `From<crossterm::event::Event>` to map:
- `Event::FocusGained` -> `FlowrsEvent::FocusGained`
- `Event::FocusLost` -> `FlowrsEvent::FocusLost`

### 3. App State (`src/app/state.rs`)

Add a boolean field to track focus:

```rust
pub struct App {
    // ... existing fields ...
    pub focused: bool,  // defaults to true
}
```

### 4. Main Loop (`src/app.rs`)

Handle focus events early in the event loop and filter Ticks when unfocused:

```rust
if let Some(event) = events.next().await {
    // Handle focus changes first
    match &event {
        FlowrsEvent::FocusGained => {
            app.lock().unwrap().focused = true;
            continue;
        }
        FlowrsEvent::FocusLost => {
            app.lock().unwrap().focused = false;
            continue;
        }
        _ => {}
    }

    // Skip tick processing when unfocused
    if let FlowrsEvent::Tick = &event {
        if !app.lock().unwrap().focused {
            continue;
        }
    }

    // ... rest of existing event handling ...
}
```

### 5. UI Indicator (Optional)

Since `focused` is in App state, a visual indicator can be shown when paused (e.g., dimmed UI, "Paused" text in status bar, or different border color).

## File Summary

| File | Change |
|------|--------|
| `src/main.rs` | Enable/disable focus reporting in terminal setup/teardown |
| `src/app/events/custom.rs` | Add `FocusGained`/`FocusLost` variants + From impl |
| `src/app/state.rs` | Add `focused: bool` field to App |
| `src/app.rs` | Handle focus events, skip Ticks when unfocused |
| `src/ui.rs` (optional) | Show paused indicator |

## Compatibility

Most modern terminals support focus reporting (iTerm2, Kitty, Windows Terminal, Alacritty, etc.). Terminals that don't support it will simply ignore the escape sequences - the app continues to work normally, just without the pause-on-unfocus feature.

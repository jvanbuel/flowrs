# Vim-Style Visual Selection Mode

## Overview

Replace the current `Ctrl+M` / `M` marking mechanism with vim-style visual selection mode for selecting multiple DAGRuns and TaskInstances.

## Motivation

The current multi-selection approach using `M` to toggle individual marks is cumbersome. Users familiar with vim expect to be able to:
- Press `V` to enter visual mode
- Navigate with `j`/`k`/`G`/`gg` to select a range
- Act on the selection with `m`

## Design

### Core Data Model

**New state fields in `DagRunModel` and `TaskInstanceModel`:**

```rust
struct DagRunModel {
    // ... existing fields ...

    // Remove: marked: Vec<usize>

    // Add:
    visual_mode: bool,            // Whether visual mode is active
    visual_anchor: Option<usize>, // The row where V was pressed
}
```

**Selection computation:**
- Selection = all indices between `visual_anchor` and `current_cursor` (inclusive)
- No need to store a `Vec<usize>` — the selection is always a contiguous range
- Helper method: `fn visual_selection(&self) -> Option<RangeInclusive<usize>>`

### State Transitions

**Entering visual mode (`V`):**
```
Normal Mode → V pressed → Visual Mode
- Set visual_mode = true
- Set visual_anchor = current cursor position
- Current row is now selected (anchor == cursor)
```

**Navigation in visual mode (`j`, `k`, `G`, `gg`):**
```
Visual Mode → j/k/G/gg pressed → Visual Mode (selection updates)
- Move cursor normally (same as normal mode navigation)
- Selection auto-updates: range from anchor to new cursor position
- Anchor stays fixed
```

**Exiting visual mode:**
```
Visual Mode → Esc pressed → Normal Mode (cancel)
- Set visual_mode = false
- Set visual_anchor = None
- No selection, cursor stays where it is

Visual Mode → m pressed → Mark Popup (act on selection)
- Compute selected indices from anchor..=cursor
- Pass selected item IDs to mark popup
- After popup closes: visual_mode = false, visual_anchor = None
```

**`m` in normal mode (no visual selection):**
```
Normal Mode → m pressed → Mark Popup
- Pass current cursor item to mark popup (single item)
- After popup closes: cursor stays, no state change needed
```

### Selection Direction Behavior

When moving past the anchor point, selection contracts then expands (standard vim behavior):

Example: Anchor at row 5, cursor moves to row 3 (rows 3-5 selected), then cursor moves to row 7:
- Result: rows 5-7 selected (anchor always included)

### UI Rendering

**Row highlighting:**
- Selected rows get yellow background (`MARKED_COLOR`)
- Same visual treatment as current marking

**Bottom border indicator (only in visual mode):**
```rust
let block = Block::default()
    .border_type(BorderType::Rounded)
    .borders(Borders::ALL)
    .title("DAG Runs");

let block = if self.visual_mode {
    let count = self.visual_selection_count();
    block.title_bottom(format!(" -- VISUAL ({count} selected) -- "))
} else {
    block
};
```

### Helper Methods

```rust
impl DagRunModel {
    /// Returns the inclusive range of selected indices, if in visual mode
    fn visual_selection(&self) -> Option<RangeInclusive<usize>> {
        if !self.visual_mode {
            return None;
        }
        let anchor = self.visual_anchor?;
        let cursor = self.filtered.state.selected()?;
        let (start, end) = if anchor <= cursor {
            (anchor, cursor)
        } else {
            (cursor, anchor)
        };
        Some(start..=end)
    }

    /// Returns count of selected items
    fn visual_selection_count(&self) -> usize {
        self.visual_selection()
            .map(|r| r.end() - r.start() + 1)
            .unwrap_or(0)
    }

    /// Returns selected item IDs for passing to mark popup
    fn selected_item_ids(&self) -> Vec<String> {
        match self.visual_selection() {
            Some(range) => range
                .filter_map(|i| self.filtered.items.get(i))
                .map(|item| item.dag_run_id.clone())
                .collect(),
            None => {
                // Normal mode: just current item
                self.filtered.state.selected()
                    .and_then(|i| self.filtered.items.get(i))
                    .map(|item| vec![item.dag_run_id.clone()])
                    .unwrap_or_default()
            }
        }
    }
}
```

### Keybinding Summary

| Key | Normal Mode | Visual Mode |
|-----|-------------|-------------|
| `V` | Enter visual mode | — |
| `j`/`k` | Move cursor | Extend/contract selection |
| `G` | Jump to last | Extend selection to last |
| `gg` | Jump to first | Extend selection to first |
| `m` | Mark current item | Mark all selected items |
| `Esc` | — | Exit visual mode, clear selection |

**Removed:** `M` keybinding (toggle individual marks)

## Files to Modify

- `src/app/model/dagruns.rs` — Add visual mode state and logic
- `src/app/model/taskinstances.rs` — Add visual mode state and logic
- `src/app/model/popup/dagruns/commands.rs` — Update help text
- `src/app/model/popup/taskinstances/commands.rs` — Update help text

## Applies To

- DAGRuns panel
- TaskInstances panel

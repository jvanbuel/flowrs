# Tabs Navigation Design

## Overview

Add a visual tab bar for navigating between panels (Config, DAGs, Runs, Tasks, Logs) in the Flowrs TUI. The tabs provide a clear indicator of the current panel while maintaining existing keyboard navigation.

## Visual Design

The tab bar uses a Lip Gloss-inspired style where the active tab visually "opens" into the content below:

```
╭──────────╮╭─────────╮╭────────╮╭─────────╮╭────────╮
│ ⚙ Config ││ 𖣘 DAGs  ││ ▶ Runs ││ ◉ Tasks ││ ≣ Logs │
├──────────┴┘         └┴────────┴┴─────────┴┴────────┴───────╮
│                                                            │
│                    Active Panel Table                      │
│                                                            │
╰────────────────────────────────────────────────────────────╯
```

### Key Visual Elements

- **All tabs**: Three-sided border (top, left, right) using rounded corners (`╭ ╮`)
- **Inactive tabs**: Bottom border with `┴` connectors
- **Active tab**: No bottom border - left side uses `┘`, right side uses `└`, leaving a gap
- **First tab special case**:
  - If active: left edge uses `│` (vertical only, no horizontal into gap)
  - If inactive: left edge uses `├` (T-junction connecting to panel border)
- **Shared border line**: Extends from tabs to panel edges, ends with `╮` (rounded)
- **Content panel**: Only has side borders and rounded bottom corners (no top border)

### Tab Labels

| Panel | Label |
|-------|-------|
| Config | `⚙ Config` |
| Dag | `𖣘 DAGs` |
| DAGRun | `▶ Runs` |
| TaskInstance | `◉ Tasks` |
| Logs | `≣ Logs` |

### Styling

- **Active tab**: `PURPLE` background with `TEXT_PRIMARY` foreground
- **Inactive tabs**: `BORDER_STYLE` (dimmed purple) borders, default text
- **Panel borders**: Rounded bottom corners with `BorderType::Rounded`
- **Panel titles**: Show "Press <?> to see available commands"

## Layout Structure

```
┌─────────────────────────────────────────┐
│  Flowrs v0.x.x            Fetching...   │  ← Header (1 line)
├─────────────────────────────────────────┤
│ ╭─────╮╭─────╮╭─────╮╭─────╮╭─────╮     │
│ │ Tab ││ Tab ││ Tab ││ Tab ││ Tab │     │  ← Tab bar (3 lines)
│ ├─────┴┘     └┴─────┴┴─────┴┴─────┴────╮│
├─────────────────────────────────────────┤
│  Press <?> to see available commands    │
│           Active Panel Table            │  ← Panel content (remaining)
│                                         │
╰─────────────────────────────────────────╯
```

**Vertical allocation:**
- Header: 1 line (unchanged)
- Tab bar: 3 lines (tab tops, tab content, shared border)
- Panel: remaining space

## Behavior

**Purely visual** - the tabs indicate current position but don't change navigation:
- `Enter` / `Right` / `l`: Move to next panel
- `Esc` / `Left` / `h`: Move to previous panel

No new keyboard shortcuts or click handling.

## Implementation

### New Component: `TabBar` Widget

Created `src/ui/tabs.rs` with a custom `TabBar` widget:

1. Defines `Tab` struct with icon and label
2. `TABS` constant array with all five panel tabs
3. `TabBar` widget that renders:
   - Line 1: Tab tops (`╭───╮`)
   - Line 2: Tab content (`│ icon label │`)
   - Line 3: Shared border line with proper connectors
4. Handles active vs inactive tab styling
5. Extends border line to panel edges with rounded corner (`╮`)

### Border Connection Characters

| Position | Active Tab | Inactive Tab |
|----------|------------|--------------|
| First tab left | `│` | `├` |
| Other tab left | `┘` | `┴` |
| Tab right | `└` | `┴` |
| Right edge | `╮` | `╮` |

### Changes to Existing Code

**`src/ui.rs`:**
- Added `pub mod tabs`
- Import `TabBar` and `TAB_BAR_HEIGHT`
- Layout split: header (1) + tab bar (3) + panel (remaining)
- Render `TabBar` with active panel index

**Panel models (`src/app/model/*.rs`):**
- Changed `Borders::ALL` to `Borders::LEFT | Borders::RIGHT | Borders::BOTTOM`
- Changed `BorderType::Plain` to `BorderType::Rounded`
- Updated titles to show "Press <?> to see available commands"
- Removed redundant panel names from titles (now shown in tabs)

### Why Custom Widget

Ratatui's built-in `Tabs` widget doesn't support:
- Per-tab borders (only a divider between tabs)
- The shared border line with selective gaps
- Three-sided tab styling with proper connection characters

Custom rendering gives full control over the visual effect.

## Edge Cases

- **Narrow terminal**: Tabs stop rendering if they exceed available width
- **Terminal font**: Icons (`⚙ 𖣘 ▶ ◉ ≣`) require Unicode support (standard in modern terminals)

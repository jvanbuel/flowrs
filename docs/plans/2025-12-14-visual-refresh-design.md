# Visual Refresh Design

## Overview

A comprehensive visual overhaul for Flowrs TUI, creating a polished dark theme with modern aesthetics inspired by lipgloss/BubbleTea. Focus on one beautiful default theme rather than a theming system.

## Design Goals

- Modern, minimal, sharp aesthetic
- Clear visual hierarchy through color and spacing
- Refined selection states (no more simple color inversion)
- Tighter, more intentional layout

## Color Palette

### Primary Colors

| Role | Color | Hex | Usage |
|------|-------|-----|-------|
| Background | Deep charcoal | `#1A1B26` | Main app background |
| Surface | Soft charcoal | `#24253A` | Panels, alternating rows, popup backgrounds |
| Primary | Rich purple | `#7663E7` | Default text, borders, structural elements |
| Primary muted | Soft lavender | `#9D8FD9` | Secondary text, disabled states |
| Accent | Emerald | `#00C896` | Selections, focus states, interactive highlights |

### Semantic Colors (Airflow States)

| State | Color | Hex |
|-------|-------|-----|
| Success | Green | `#50C878` |
| Failed | Coral red | `#FF6B6B` |
| Running | Bright cyan | `#22FFAA` |
| Queued | Muted gray | `#808080` |
| Up for retry | Amber | `#FFB347` |

Note: Success state (`#50C878`) differs from UI accent (`#00C896`) to avoid confusion between "selected" and "successful."

## Selection States & Buttons

### Button States (Popups)

| State | Background | Border | Text |
|-------|------------|--------|------|
| Default | Transparent | 1px `#7663E7` | `#7663E7` |
| Focused/Selected | `#00C89620` (emerald 12% opacity) | 2px `#00C896` | `#00C896` |
| Disabled | Transparent | 1px `#4A4A5A` | `#4A4A5A` |

Selected state combines:
- Subtle emerald background tint
- Thicker, brighter emerald border
- Text color shifts to emerald

### Table Row Selection

| State | Background | Left indicator |
|-------|------------|----------------|
| Default | `#1A1B26` | None |
| Alternating | `#24253A` | None |
| Selected | `#00C89615` (emerald 8% opacity) | 2px solid emerald bar |
| Marked (visual selection mode) | `#7663E715` (purple 8% opacity) | Small purple square |

## Spacing & Layout

### Popups
- Width: ~35% (down from 50%) for simple dialogs
- Height: Auto-sized based on content
- Padding: 1 line vertical, 2 chars horizontal inside border
- Position: Centered vertically and horizontally

### Table Layout
- Row height: 1 line with visual breathing room via alternating backgrounds
- Column padding: 1 space minimum between columns
- Header row: Subtle bottom border (`─`) in primary purple

### Headers & Title Bar
- Top bar: Solid surface color (`#24253A`) background
- Panel title: Bold text + bottom border line
- Help hint: Muted lavender (`#9D8FD9`)

### Border Styles
- Popups/modals: Rounded corners (`BorderType::Rounded`)
- Tables/panels: Sharp corners (`BorderType::Plain`)

## Implementation Approach

### New Theme Module

Create `src/ui/theme.rs` to centralize all styling:

```rust
pub struct Theme {
    pub bg: Color,
    pub surface: Color,
    pub primary: Color,
    pub primary_muted: Color,
    pub accent: Color,
    // ... semantic colors
}

pub static THEME: LazyLock<Theme> = LazyLock::new(|| Theme::default());
```

Replace scattered `DEFAULT_STYLE` usage with semantic accessors like `THEME.button_selected()`, `THEME.row_alt()`.

### Files to Modify

| File | Changes |
|------|---------|
| `src/ui/constants.rs` | Replace `DEFAULT_STYLE` and colors with new `Theme` |
| `src/ui/theme.rs` | New file - theme struct and style builders |
| `src/app/model/popup/*.rs` | Update button rendering for new selection styles |
| `src/app/model/*.rs` | Update table row rendering for new selection/alternating styles |
| `src/ui.rs` | Update header bar rendering |

### Migration Strategy
1. Create theme module with all colors/styles defined
2. Migrate one component at a time (start with popups)
3. Keep backward compatibility during migration

## Out of Scope

- Multiple theme support / theme switching
- Light mode
- User-configurable colors

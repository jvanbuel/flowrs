# Vim-Style Visual Selection Mode Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the `M` marking mechanism with vim-style `V` visual selection mode for DAGRuns and TaskInstances panels.

**Architecture:** Add `visual_mode: bool` and `visual_anchor: Option<usize>` fields to both models. Selection is computed as the range between anchor and cursor. Navigation keys extend/contract the selection. `Esc` cancels, `m` acts on selection.

**Tech Stack:** Rust, ratatui (Block::title_bottom for indicator), crossterm (KeyCode::Char('V'))

---

## Task 1: Add Visual Mode State to DagRunModel

**Files:**
- Modify: `src/app/model/dagruns.rs:33-45` (struct definition)
- Modify: `src/app/model/dagruns.rs:69-83` (new() constructor)

**Step 1: Add visual mode fields to struct**

In `src/app/model/dagruns.rs`, replace the `marked` field with visual mode fields:

```rust
pub struct DagRunModel {
    pub dag_id: Option<String>,
    pub dag_code: DagCodeWidget,
    pub all: Vec<DagRun>,
    pub filtered: StatefulTable<DagRun>,
    pub filter: Filter,
    pub visual_mode: bool,
    pub visual_anchor: Option<usize>,
    pub popup: Option<DagRunPopUp>,
    pub commands: Option<&'static CommandPopUp<'static>>,
    pub error_popup: Option<ErrorPopup>,
    ticks: u32,
    event_buffer: Vec<FlowrsEvent>,
}
```

**Step 2: Update the constructor**

In `DagRunModel::new()`, replace `marked: vec![]` with the new fields:

```rust
impl DagRunModel {
    pub fn new() -> Self {
        DagRunModel {
            dag_id: None,
            dag_code: DagCodeWidget::default(),
            all: vec![],
            filtered: StatefulTable::new(vec![]),
            filter: Filter::new(),
            visual_mode: false,
            visual_anchor: None,
            popup: None,
            commands: None,
            error_popup: None,
            ticks: 0,
            event_buffer: vec![],
        }
    }
```

**Step 3: Build to verify no compile errors**

Run: `cargo build 2>&1 | head -50`
Expected: Compile errors about `marked` being used elsewhere (this is expected, we'll fix in later tasks)

---

## Task 2: Add Helper Methods to DagRunModel

**Files:**
- Modify: `src/app/model/dagruns.rs` (add methods after `current()` around line 185)

**Step 1: Add the use statement for RangeInclusive**

At the top of the file, add:

```rust
use std::ops::RangeInclusive;
```

**Step 2: Add visual selection helper methods**

After the `current()` method (around line 185), add:

```rust
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

    /// Returns count of selected items (for bottom border display)
    fn visual_selection_count(&self) -> usize {
        self.visual_selection()
            .map(|r| r.end() - r.start() + 1)
            .unwrap_or(0)
    }

    /// Returns selected DAG run IDs for passing to mark popup
    fn selected_dag_run_ids(&self) -> Vec<String> {
        match self.visual_selection() {
            Some(range) => range
                .filter_map(|i| self.filtered.items.get(i))
                .map(|item| item.dag_run_id.clone())
                .collect(),
            None => {
                // Normal mode: just current item
                self.filtered
                    .state
                    .selected()
                    .and_then(|i| self.filtered.items.get(i))
                    .map(|item| vec![item.dag_run_id.clone()])
                    .unwrap_or_default()
            }
        }
    }
```

**Step 3: Build to check syntax**

Run: `cargo build 2>&1 | head -50`
Expected: Still compile errors about `marked` (expected)

---

## Task 3: Update DagRunModel Keybindings

**Files:**
- Modify: `src/app/model/dagruns.rs:308-406` (key handling in update())

**Step 1: Add `V` keybinding to enter visual mode**

In the main keybinding match block (after line 329, after the `gg` handling), add:

```rust
                        KeyCode::Char('V') => {
                            if let Some(cursor) = self.filtered.state.selected() {
                                self.visual_mode = true;
                                self.visual_anchor = Some(cursor);
                            }
                        }
```

**Step 2: Update navigation keys to work in visual mode**

The existing `j`, `k`, `G`, `gg` keys already move the cursor. No changes needed - visual selection auto-updates based on anchor and cursor position.

**Step 3: Add `Esc` handler to exit visual mode**

In the main keybinding match, add a case for `Esc` (before the existing handlers):

```rust
                        KeyCode::Esc => {
                            if self.visual_mode {
                                self.visual_mode = false;
                                self.visual_anchor = None;
                                return (None, vec![]); // Consume event when exiting visual mode
                            }
                            // Propagate Esc to navigate back to previous panel
                            return (Some(FlowrsEvent::Key(*key_event)), vec![]);
                        }
```

**Step 4: Update `m` keybinding to use visual selection**

Replace the existing `m` handler (lines 335-346) with:

```rust
                        KeyCode::Char('m') => {
                            let dag_run_ids = self.selected_dag_run_ids();
                            if !dag_run_ids.is_empty() {
                                self.popup = Some(DagRunPopUp::Mark(MarkDagRunPopup::new(
                                    dag_run_ids,
                                    self.dag_id.clone().unwrap_or_default(),
                                )));
                            }
                        }
```

**Step 5: Remove the `M` keybinding handler**

Delete lines 348-356 (the `KeyCode::Char('M')` handler).

**Step 6: Update popup close handler to clear visual mode**

In the `DagRunPopUp::Mark` handler (around line 256-268), update to clear visual mode:

```rust
                        DagRunPopUp::Mark(popup) => {
                            let (key_event, messages) = popup.update(event);
                            debug!("Popup messages: {messages:?}");
                            if let Some(FlowrsEvent::Key(key_event)) = &key_event {
                                match key_event.code {
                                    KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                                        self.popup = None;
                                        self.visual_mode = false;
                                        self.visual_anchor = None;
                                    }
                                    _ => {}
                                }
                            }
                            return (None, messages);
                        }
```

**Step 7: Build to verify**

Run: `cargo build 2>&1 | head -50`
Expected: Fewer errors, remaining errors about `marked` in render

---

## Task 4: Update DagRunModel Rendering

**Files:**
- Modify: `src/app/model/dagruns.rs:462-534` (render method)

**Step 1: Update row highlighting to use visual selection**

Replace lines 503-509:

```rust
            .style(if self.marked.contains(&idx) {
                DEFAULT_STYLE.bg(MARKED_COLOR)
            } else if (idx % 2) == 0 {
                DEFAULT_STYLE
            } else {
                DEFAULT_STYLE.bg(ALTERNATING_ROW_COLOR)
            })
```

With:

```rust
            .style(
                if self
                    .visual_selection()
                    .map_or(false, |r| r.contains(&idx))
                {
                    DEFAULT_STYLE.bg(MARKED_COLOR)
                } else if (idx % 2) == 0 {
                    DEFAULT_STYLE
                } else {
                    DEFAULT_STYLE.bg(ALTERNATING_ROW_COLOR)
                },
            )
```

**Step 2: Add visual mode indicator to bottom border**

Replace the block creation (lines 523-533):

```rust
        .block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title(if let Some(dag_id) = &self.dag_id {
                    format!("DAGRuns ({dag_id}) - press <?> to see available commands")
                } else {
                    "DAGRuns".to_string()
                })
                .style(DEFAULT_STYLE),
        )
```

With:

```rust
        .block({
            let block = Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title(if let Some(dag_id) = &self.dag_id {
                    format!("DAGRuns ({dag_id}) - press <?> to see available commands")
                } else {
                    "DAGRuns".to_string()
                })
                .style(DEFAULT_STYLE);
            if self.visual_mode {
                block.title_bottom(format!(
                    " -- VISUAL ({} selected) -- ",
                    self.visual_selection_count()
                ))
            } else {
                block
            }
        })
```

**Step 3: Build to verify DagRunModel compiles**

Run: `cargo build 2>&1 | head -50`
Expected: DagRunModel should compile, errors only from TaskInstanceModel

---

## Task 5: Add Visual Mode State to TaskInstanceModel

**Files:**
- Modify: `src/app/model/taskinstances.rs:27-39` (struct definition)
- Modify: `src/app/model/taskinstances.rs:42-56` (new() constructor)

**Step 1: Add use statement for RangeInclusive**

At the top of the file, add:

```rust
use std::ops::RangeInclusive;
```

**Step 2: Add visual mode fields to struct**

Replace the `marked` field:

```rust
pub struct TaskInstanceModel {
    pub dag_id: Option<String>,
    pub dag_run_id: Option<String>,
    pub all: Vec<TaskInstance>,
    pub filtered: StatefulTable<TaskInstance>,
    pub filter: Filter,
    pub popup: Option<TaskInstancePopUp>,
    pub visual_mode: bool,
    pub visual_anchor: Option<usize>,
    commands: Option<&'static CommandPopUp<'static>>,
    pub error_popup: Option<ErrorPopup>,
    ticks: u32,
    event_buffer: Vec<FlowrsEvent>,
}
```

**Step 3: Update the constructor**

Replace `marked: vec![]` with:

```rust
    pub fn new() -> Self {
        TaskInstanceModel {
            dag_id: None,
            dag_run_id: None,
            all: vec![],
            filtered: StatefulTable::new(vec![]),
            filter: Filter::new(),
            popup: None,
            visual_mode: false,
            visual_anchor: None,
            commands: None,
            error_popup: None,
            ticks: 0,
            event_buffer: vec![],
        }
    }
```

**Step 4: Build to check**

Run: `cargo build 2>&1 | head -50`
Expected: Compile errors about `marked` being used elsewhere in TaskInstanceModel

---

## Task 6: Add Helper Methods to TaskInstanceModel

**Files:**
- Modify: `src/app/model/taskinstances.rs` (add methods after `mark_task_instance()` around line 86)

**Step 1: Add visual selection helper methods**

After `mark_task_instance()`, add:

```rust
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

    /// Returns count of selected items (for bottom border display)
    fn visual_selection_count(&self) -> usize {
        self.visual_selection()
            .map(|r| r.end() - r.start() + 1)
            .unwrap_or(0)
    }

    /// Returns selected task IDs for passing to mark popup
    fn selected_task_ids(&self) -> Vec<String> {
        match self.visual_selection() {
            Some(range) => range
                .filter_map(|i| self.filtered.items.get(i))
                .map(|item| item.task_id.clone())
                .collect(),
            None => {
                // Normal mode: just current item
                self.filtered
                    .state
                    .selected()
                    .and_then(|i| self.filtered.items.get(i))
                    .map(|item| vec![item.task_id.clone()])
                    .unwrap_or_default()
            }
        }
    }
```

**Step 2: Build to check syntax**

Run: `cargo build 2>&1 | head -50`
Expected: Still compile errors about `marked` (expected)

---

## Task 7: Update TaskInstanceModel Keybindings

**Files:**
- Modify: `src/app/model/taskinstances.rs:165-263` (key handling in update())

**Step 1: Add `V` keybinding to enter visual mode**

After the `gg` handler (after line 186), add:

```rust
                        KeyCode::Char('V') => {
                            if let Some(cursor) = self.filtered.state.selected() {
                                self.visual_mode = true;
                                self.visual_anchor = Some(cursor);
                            }
                        }
```

**Step 2: Add `Esc` handler to exit visual mode**

Before the existing match arms, add:

```rust
                        KeyCode::Esc => {
                            if self.visual_mode {
                                self.visual_mode = false;
                                self.visual_anchor = None;
                                return (None, vec![]);
                            }
                            return (Some(FlowrsEvent::Key(*key_event)), vec![]);
                        }
```

**Step 3: Update `m` keybinding to use visual selection**

Replace the existing `m` handler (lines 187-203) with:

```rust
                        KeyCode::Char('m') => {
                            let task_ids = self.selected_task_ids();
                            if !task_ids.is_empty() {
                                if let (Some(dag_id), Some(dag_run_id)) =
                                    (&self.dag_id, &self.dag_run_id)
                                {
                                    self.popup =
                                        Some(TaskInstancePopUp::Mark(MarkTaskInstancePopup::new(
                                            task_ids,
                                            dag_id,
                                            dag_run_id,
                                        )));
                                }
                            }
                        }
```

**Step 4: Remove the `M` keybinding handler**

Delete lines 205-213 (the `KeyCode::Char('M')` handler).

**Step 5: Update popup close handler to clear visual mode**

In the `TaskInstancePopUp::Mark` handler (around line 150-163), update:

```rust
                        TaskInstancePopUp::Mark(popup) => {
                            let (key_event, messages) = popup.update(event);
                            debug!("Popup messages: {messages:?}");
                            if let Some(FlowrsEvent::Key(key_event)) = &key_event {
                                match key_event.code {
                                    KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                                        self.popup = None;
                                        self.visual_mode = false;
                                        self.visual_anchor = None;
                                    }
                                    _ => {}
                                }
                            }
                            return (None, messages);
                        }
```

**Step 6: Build to verify**

Run: `cargo build 2>&1 | head -50`
Expected: Fewer errors, remaining errors about `marked` in render

---

## Task 8: Update TaskInstanceModel Rendering

**Files:**
- Modify: `src/app/model/taskinstances.rs:294-354` (render method)

**Step 1: Update row highlighting to use visual selection**

Replace lines 326-332:

```rust
            .style(if self.marked.contains(&idx) {
                DEFAULT_STYLE.bg(MARKED_COLOR)
            } else if (idx % 2) == 0 {
                DEFAULT_STYLE
            } else {
                DEFAULT_STYLE.bg(ALTERNATING_ROW_COLOR)
            })
```

With:

```rust
            .style(
                if self
                    .visual_selection()
                    .map_or(false, |r| r.contains(&idx))
                {
                    DEFAULT_STYLE.bg(MARKED_COLOR)
                } else if (idx % 2) == 0 {
                    DEFAULT_STYLE
                } else {
                    DEFAULT_STYLE.bg(ALTERNATING_ROW_COLOR)
                },
            )
```

**Step 2: Add visual mode indicator to bottom border**

Replace the block creation (lines 345-350):

```rust
        .block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title("TaskInstances - Press <?> to see available commands"),
        )
```

With:

```rust
        .block({
            let block = Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title("TaskInstances - Press <?> to see available commands");
            if self.visual_mode {
                block.title_bottom(format!(
                    " -- VISUAL ({} selected) -- ",
                    self.visual_selection_count()
                ))
            } else {
                block
            }
        })
```

**Step 3: Build to verify full compilation**

Run: `cargo build`
Expected: SUCCESS - no compile errors

---

## Task 9: Update Command Help Documentation

**Files:**
- Modify: `src/app/model/popup/dagruns/commands.rs`
- Modify: `src/app/model/popup/taskinstances/commands.rs`

**Step 1: Update DagRun commands**

Replace the commands array in `src/app/model/popup/dagruns/commands.rs`:

```rust
pub static DAGRUN_COMMAND_POP_UP: LazyLock<CommandPopUp> = LazyLock::new(|| {
    let mut commands = vec![
        Command {
            name: "Clear",
            key_binding: "c",
            description: "Clear a DAG run",
        },
        Command {
            name: "Show",
            key_binding: "v",
            description: "Show DAG code",
        },
        Command {
            name: "Visual",
            key_binding: "V",
            description: "Enter visual selection mode",
        },
        Command {
            name: "Mark",
            key_binding: "m",
            description: "Mark selected DAG run(s)",
        },
        Command {
            name: "Trigger",
            key_binding: "t",
            description: "Trigger a DAG run",
        },
    ];
    commands.append(&mut DefaultCommands::new().0);
    CommandPopUp {
        title: "DAG Run Commands".into(),
        commands,
    }
});
```

**Step 2: Update TaskInstance commands**

Replace the commands array in `src/app/model/popup/taskinstances/commands.rs`:

```rust
pub static TASK_COMMAND_POP_UP: LazyLock<CommandPopUp> = LazyLock::new(|| {
    let mut commands = vec![
        Command {
            name: "Clear",
            key_binding: "c",
            description: "Clear a task instance",
        },
        Command {
            name: "Visual",
            key_binding: "V",
            description: "Enter visual selection mode",
        },
        Command {
            name: "Mark",
            key_binding: "m",
            description: "Mark selected task instance(s)",
        },
        Command {
            name: "Filter",
            key_binding: "/",
            description: "Filter task instances",
        },
    ];

    commands.append(&mut DefaultCommands::new().0);
    CommandPopUp {
        title: "Task Commands".into(),
        commands,
    }
});
```

**Step 3: Build to verify**

Run: `cargo build`
Expected: SUCCESS

---

## Task 10: Run Clippy and Fix Warnings

**Files:**
- Potentially any modified files

**Step 1: Run clippy**

Run: `cargo clippy 2>&1`
Expected: Review any warnings related to the changes

**Step 2: Fix any clippy warnings**

Address any warnings about unused imports, unnecessary clones, etc.

**Step 3: Run clippy again to verify clean**

Run: `cargo clippy 2>&1`
Expected: No warnings related to visual mode changes

---

## Task 11: Manual Testing

**Step 1: Run the application**

Run: `FLOWRS_LOG=debug cargo run`

**Step 2: Test visual mode in DAGRuns panel**

1. Navigate to a DAG with multiple runs
2. Press `V` to enter visual mode - verify bottom border shows `-- VISUAL (1 selected) --`
3. Press `j` multiple times - verify count increases and rows highlight yellow
4. Press `k` - verify count decreases
5. Press `G` - verify selection extends to last item
6. Press `gg` - verify selection contracts and extends to first item
7. Press `Esc` - verify visual mode exits and selection clears
8. Press `V`, navigate, press `m` - verify mark popup opens with selected items

**Step 3: Test visual mode in TaskInstances panel**

Repeat the same tests in the TaskInstances panel.

**Step 4: Test single item marking**

1. In normal mode (not visual mode), press `m`
2. Verify mark popup opens with just the current item

**Step 5: Verify M keybinding is removed**

1. Press `M` in both panels
2. Verify nothing happens (no toggle marking)

---

## Task 12: Commit Changes

**Step 1: Review changes**

Run: `jj diff`

**Step 2: Commit**

Run: `jj commit -m "feat: replace M marking with V visual selection mode"`

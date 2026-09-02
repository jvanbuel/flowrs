# Data-oriented design in flowrs

Reference: Richard Fabian, *Data-Oriented Design* (dataorienteddesign.com/dodbook).
The book's thesis: the program is a set of transforms over data; design the
data for the transform that runs most often, and treat objects, hierarchies
and per-item branching as costs to justify rather than defaults.

## Where the hot paths are in this repo

| Transform | Frequency | Code |
|---|---|---|
| Draw a panel | every tick (5 Hz) and every key | `src/app/model/*/render.rs`, `src/ui/gantt.rs` |
| Filter a table | every keystroke in the filter, every poll | `src/app/model/filter/matching.rs` |
| Sync cache to panel | every poll (2 s) | `src/app/state/sync.rs` |
| Build the task graph | on entering a DAG run | `src/airflow/graph.rs` |

Anything that runs inside the first two rows is per item per frame. Measure
work there in "allocations per row", not in big-O.

## Checklist (Fabian chapter in brackets)

1. **Tables, not object graphs** [Relational model]. Keep collections flat
   and keyed by a dense index. `TaskGraph` is the pattern: `ids[i]`,
   `level_of[i]`, CSR `edge_starts`/`edges`, and one `HashMap` only at the
   boundary where names arrive from outside.
2. **Prepare data for the transform, not in it** [Data-oriented
   transforms]. Normalise once at construction: `FilterCondition` lowercases
   its needle when built, not per item. State enums expose `as_str()` so
   display and filter compare a `&'static str` instead of formatting.
3. **Static tables for metadata** [Condition tables]. Field lists are
   `const` slices (`impl_filterable!`), never a `Vec` rebuilt per key press.
4. **Borrow in the hot loop; own at the write** [Existence-based
   processing, memory]. `FilterableTable::rows_and_state` splits the items
   from the `TableState` so `Row`s borrow cell text. `Filterable::get_field_value`
   returns `Cow` and borrows whenever the field is already text.
5. **Sort keys, not payloads** [Sorting]. Compute a rank once per element
   (`sort_by_cached_key`), never hash inside the comparator.
6. **Hoist invariants out of the row loop** [Optimisations]. One
   `OffsetDateTime::now_utc()` per frame, passed down; one `Style` per
   column, not per cell.
7. **Slices over scans** [Searching]. If a query is "all X at level L",
   store the data so the answer is a range, not a filter over everything.

## When reviewing a change, ask

- Does any closure inside `.map(|(idx, item)| ...)` in a render call
  `to_string()`, `clone()`, `format!` or a `HashMap` lookup that could have
  been done when the data last changed?
- Is a `HashMap<String, _>` keyed by something that already has a dense
  index?
- Is a per-item `Option`/`bool` checked in a loop when the items could have
  been partitioned once?
- Would the transform be simpler if the data were laid out in the order it
  is consumed?

## Known, deliberately unchanged

- `sync_panel` still clones `Vec<Dag>` / `Vec<DagRun>` out of the
  environment cache per poll. Sharing via `Arc` needs copy-on-write on the
  worker side; do it only with a measurement showing it matters.
- Every filtered row is still built per frame; ratatui's `Table` clips to
  the viewport. Windowing rows means owning scroll-offset logic.
- `GanttData` keys tries by `TaskId` in a `HashMap`; one lookup per row is
  acceptable at current table sizes.

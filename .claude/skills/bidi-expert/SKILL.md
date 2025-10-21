---
name: bidi-expert
description: Implements Right-to-Left (RTL) and Bidirectional text in terminal emulators using terminal-wg spec and Unicode BiDi Algorithm (UAX#9). Handles Hebrew/Arabic rendering, logical↔visual reordering, escape sequences (CSI 8, SCP, 2500/2501/1243), position mapping, cursor movement, and display transformation. Use for BiDi features, RTL debugging, character positioning, mixed LTR/RTL bugs, or embedding level issues.
---

# BiDi Expert for Alacritty

Expert guidance for implementing Right-to-Left (RTL) and Bidirectional text support in terminal emulators according to terminal-wg specification and Unicode BiDi Algorithm (UAX#9).

## Quick Reference Cards

### When BiDi is Needed

```
✅ YES: Hebrew/Arabic text ("שלום", "مرحبا")
✅ YES: Mixed RTL/LTR ("hello שלום world")
✅ YES: RTL with numbers ("שלום 123")
❌ NO:  Pure LTR text ("hello world")
❌ NO:  Empty lines or whitespace-only
```

### Escape Sequences Quick Table

| Sequence | Name | Purpose |
|----------|------|---------|
| `CSI 8 h` / `CSI 8 l` | BiDi Implicit Mode | Enable/disable BiDi reordering |
| `CSI 0 SP k` | SCP Default | Context-dependent direction (usually LTR) |
| `CSI 1 SP k` | SCP LTR | Force LTR paragraph |
| `CSI 2 SP k` | SCP RTL | Force RTL paragraph |
| `CSI ? 2501 h/l` | Auto-detect | Auto-detect paragraph direction |
| `CSI ? 2500 h/l` | Box Mirroring | Mirror box-drawing chars in RTL |
| `CSI ? 1243 h/l` | Arrow Swapping | Swap arrow keys in RTL mode |

### Common Test Cases

| Input | Expected Visual | Levels | Notes |
|-------|----------------|--------|-------|
| "hello" | "hello" | [0,0,0,0,0] | Identity mapping |
| "שלום" | "םולש" | [1,1,1,1] | Full reversal |
| "hello שלום world" | "hello םולש world" | [0...0, 1...1, 0...0] | Three runs |
| "שלום!" | "!םולש" | [1,1,1,1,1] | Punctuation follows RTL |

### Position Mapping Quick Check

```rust
// Logical → Visual (for cursor display)
visual_pos = logical_to_visual[logical_pos]

// Visual → Logical (for mouse clicks)
logical_pos = visual_to_logical[visual_pos]

// Build mapping using canonical algorithm
let reordered_levels = bidi_info.reordered_levels_per_char(para, range);
let visual_to_logical = BidiInfo::reorder_visual(&reordered_levels);
```

## Troubleshooting Guide

### Problem: Characters appear in wrong positions

**Diagnostic steps:**
1. Check if line has RTL chars → Use `is_rtl_char()` check
2. Verify BiDi mode enabled → Check `BidiMode::Implicit` in cells
3. Inspect embedding levels → Print levels array, verify odd=RTL, even=LTR
4. Validate position mapping → Print `logical_to_visual` array
5. Check if using canonical mapping → Must use `reorder_visual()`, not char matching

**Common causes:**
- Using character matching instead of canonical mapping
- Forgetting to handle multi-byte UTF-8
- Not trimming trailing whitespace before algorithm
- Modifying logical grid (NEVER do this)

### Problem: Colors or properties not preserved

**Diagnostic:**
1. Verify entire cells moved → Check `cell.fg`, `cell.bg`, `cell.flags`
2. Not just characters → Never copy only `cell.c`

**Fix:**
```rust
// ❌ WRONG: Copying only character
cell.c = reordered_char;

// ✅ CORRECT: Moving entire cell
cell.point.column = Column(visual_pos);
reordered_cells.push(cell);
```

### Problem: Mixed RTL/LTR text incorrect

**Diagnostic:**
1. Count runs → Print number of runs from `visual_runs()`
2. Check run boundaries → Verify each run's start/end byte indices
3. Inspect byte→char mapping → Essential for multi-byte UTF-8
4. Verify level changes → Levels should change at run boundaries

**Example debug output:**
```
Text: "hello שלום world"
Runs: 3 runs detected
  Run 0: bytes 0-6, level 0 (LTR)
  Run 1: bytes 6-14, level 1 (RTL)
  Run 2: bytes 14-19, level 0 (LTR)
```

### Problem: Display glitches or flickering

**Diagnostic:**
1. Check damage tracking → BiDi lines must be manually marked
2. Verify reordering happens after damage → Use `damage_tracker.frame().damage_line()`
3. Check iteration order → May need reverse iteration

**Fix:**
```rust
// After BiDi reordering, manually damage lines
for (line_num, _) in &reordered_lines {
    let damage = LineDamageBounds::new(*line_num, 0, num_cols);
    self.damage_tracker.frame().damage_line(damage);
}
```

### Problem: Performance issues

**Diagnostic:**
1. Verify fast-path check → Only run BiDi when needed
2. Check line length → Algorithm is O(n), long lines take time
3. Profile hot paths → Use `cargo flamegraph`

**Optimization:**
```rust
// Fast check before running UAX#9
fn line_needs_bidi(cells: &[Cell]) -> bool {
    cells.iter().any(|cell| {
        is_rtl_char(cell.c) || !cell.bidi_flags().is_empty()
    })
}

if !line_needs_bidi(cells) {
    return None; // Skip BiDi processing
}
```

### Decision Tree: "Which direction should I use?"

```
Is auto-detection enabled (CSI ? 2501 h)?
├─ YES → Use None for para_level (algorithm detects)
└─ NO → Is explicit SCP set (CSI 1/2 SP k)?
    ├─ YES → Use explicit level from SCP
    └─ NO → Default to LTR (Level::ltr())
```

## Core Principles

### 1. Display-Only Transformation (CRITICAL)

**The terminal grid MUST remain in logical order.**

- Applications send text in logical order
- Terminal stores text in logical order
- Terminal displays text in visual order (reordered on-the-fly)
- Cursor conversions use visual↔logical mapping

Never modify the logical grid. Only transform at display time.

### 2. Terminal-wg Conformance Level

**Level 1 Implicit Mode** - Required features:
- ✅ BiDi reordering using UAX#9
- ✅ Paragraph direction control via escape sequences
- ✅ Display-only transformation
- ✅ Escape sequence recognition
- ⚠️ Arrow key swapping (flag stored, not implemented)
- ⚠️ Box mirroring (flag stored, not rendered)

### 3. Default Paragraph Direction

**Default to LTR (level 0)** - Terminals are left-aligned by default, even when displaying RTL text.

This is counter-intuitive but required by terminal-wg spec.

## Key Concepts

### Logical vs Visual Order

- **Logical**: How characters are stored/transmitted (e.g., "שלום" = shin, lamed, vav, mem)
- **Visual**: How they appear on screen (e.g., displayed as mem, vav, lamed, shin)

### Embedding Levels

- **Even levels (0, 2, 4...)**: LTR direction
- **Odd levels (1, 3, 5...)**: RTL direction
- Each character gets a level from UAX#9 algorithm

### Level Runs

Maximal substring of characters with same embedding level.

Example: "hello שלום world"
- Run 1 (level 0): "hello "
- Run 2 (level 1): "םולש" (RTL, reversed)
- Run 3 (level 0): " world"

## Implementation Workflow

### 1. Check if Line Needs BiDi

```rust
// Fast check before running full algorithm
if !line_needs_bidi(cells) {
    return None; // No BiDi needed
}
```

### 2. Trim Trailing Cells

Per terminal-wg spec: "Ignore trailing erased cells"

```rust
while content_len > 0 && cells[content_len - 1].c == ' ' && cells[content_len - 1].flags.is_empty() {
    content_len -= 1;
}
```

### 3. Determine Paragraph Level

```rust
// Default to LTR (terminal-wg requirement)
let para_level = explicit_level.or(Some(Level::ltr()));

// Or use auto-detection if enabled
if use_auto_detect {
    para_level = None; // UAX#9 will detect from first strong char
}
```

### 4. Run UAX#9 Algorithm

```rust
let bidi_info = BidiInfo::new(&text, para_level);
let para = &bidi_info.paragraphs[0];
let (levels, runs) = bidi_info.visual_runs(para, para.range.clone());
```

### 5. Reorder by Visual Runs

```rust
for run in runs.iter() {
    let char_start = byte_to_char[run.start];
    let char_end = byte_to_char.get(run.end).copied().unwrap_or(num_cols);

    if levels[run.start].is_rtl() {
        // RTL: reverse cells
        reordered_cells.extend(cells[char_start..char_end].iter().rev().cloned());
    } else {
        // LTR: keep order
        reordered_cells.extend_from_slice(&cells[char_start..char_end]);
    }
}
```

### 6. Build Canonical Position Mapping

```rust
// Get canonical BiDi mapping (handles mixed runs correctly)
let reordered_levels = bidi_info.reordered_levels_per_char(para, para.range.clone());
let visual_to_logical = BidiInfo::reorder_visual(&reordered_levels);

// Invert to get logical→visual
let mut logical_to_visual = vec![0; num_cols];
for (visual_pos, &logical_pos) in visual_to_logical.iter().enumerate() {
    logical_to_visual[logical_pos] = visual_pos;
}
```

### 7. Apply Mapping at Display Time

```rust
// In display/mod.rs - reorder RenderableCells
for mut cell in cells {
    let logical_pos = cell.point.column.0;
    if let Some(&visual_pos) = logical_to_visual.get(logical_pos) {
        cell.point.column = Column(visual_pos);
        reordered_cells.push(cell);
    }
}
```

### 8. Mark Lines as Damaged

```rust
// BiDi happens after damage tracking, so manually mark
for (line_num, _) in &reordered_lines {
    let damage = LineDamageBounds::new(*line_num, 0, num_cols);
    self.damage_tracker.frame().damage_line(damage);
}
```

## Common Patterns

### Pattern: Multi-byte UTF-8 Handling

BiDi algorithm works on byte indices, but terminal uses char indices.

```rust
// Build byte→char mapping
let mut byte_to_char = vec![0; text.len()];
for (char_idx, (byte_idx, ch)) in text.char_indices().enumerate() {
    for i in 0..ch.len_utf8() {
        byte_to_char[byte_idx + i] = char_idx;
    }
}
```

### Pattern: Escape Sequence Storage

Store BiDi mode on cells, not globally:

```rust
// In cell.rs
pub struct Cell {
    // ... other fields
    flags: BidiFlags, // Per-cell BiDi state
}

// BidiFlags: EXPLICIT_DIRECTION, NON_DEFAULT_PARA_DIR, RTL_PARA_DIR, AUTO_PARA_DIR, BOX_MIRRORING
```

## Common Pitfalls

### ❌ NEVER: Modify the Logical Grid

```rust
// WRONG: Modifying grid for BiDi
grid[line][col].c = reordered_char;
```

### ✅ DO: Transform Only at Display

```rust
// CORRECT: Reorder RenderableCells
cell.point.column = Column(visual_pos);
```

### ❌ NEVER: Use Character Matching for Position Mapping

```rust
// WRONG: Fragile, fails on duplicates
for cell in cells {
    if cell.c == reordered[visual_pos].c {
        mapping[logical_pos] = visual_pos;
    }
}
```

### ✅ DO: Use Canonical BiDi Mapping

```rust
// CORRECT: Unambiguous
let visual_to_logical = BidiInfo::reorder_visual(&reordered_levels);
```

### ❌ NEVER: Forget Damage Tracking

BiDi reordering happens after terminal damage tracking, so you must manually mark BiDi lines.

### ❌ NEVER: Rearrange Only Characters

```rust
// WRONG: Colors/flags stay in wrong position
cell.c = reordered_char;
```

### ✅ DO: Move Complete Cells

```rust
// CORRECT: Entire cell (colors, flags, everything) moves
cell.point.column = Column(visual_pos);
reordered_cells.push(cell);
```

## Implementation Checklist

Use this checklist when implementing or debugging BiDi features:

**Basic Setup:**
- [ ] Fast-path check (`line_needs_bidi()`) before running algorithm
- [ ] Trim trailing whitespace before processing
- [ ] Determine paragraph level correctly (default LTR)
- [ ] Build byte→char mapping for UTF-8

**Reordering:**
- [ ] Use `visual_runs()` to get reordered runs
- [ ] Reverse RTL runs, keep LTR runs unchanged
- [ ] Move complete cells (character + all properties)
- [ ] Build canonical position mapping with `reorder_visual()`

**Display:**
- [ ] Apply reordering only at display time
- [ ] Never modify logical grid
- [ ] Manually mark BiDi lines as damaged
- [ ] Verify colors/hyperlinks/flags preserved

**Testing:**
- [ ] Pure LTR → identity mapping
- [ ] Pure RTL → full reversal
- [ ] Mixed RTL/LTR → correct run boundaries
- [ ] Multi-byte UTF-8 → no corruption
- [ ] Trailing spaces → ignored per spec

## Key Files Reference

- `alacritty_terminal/src/term/cell.rs:39-75` - BiDi types (BidiDir, BidiMode, BidiFlags)
- `alacritty_terminal/src/term/bidi.rs:1-333` - Core reordering logic
- `alacritty_terminal/src/term/mod.rs` - Escape sequence handlers
- `alacritty/src/display/mod.rs:840-891` - Display rendering with position mapping

## Quick Decision Tree

Need to implement BiDi feature?

**Escape sequence parsing?**
→ See `references/escape_sequences.md`
→ Store flags in BidiFlags, set BidiMode on cells

**Character reordering?**
→ See `references/uax9_algorithm.md`
→ Use unicode-bidi crate: BidiInfo::new() → visual_runs()

**Paragraph direction?**
→ Default: LTR (level 0) - terminals are left-aligned
→ Auto-detect: Use BidiInfo::new(&text, None)
→ Explicit: Use CSI SP k values

**Cursor movement?**
→ Use canonical mapping: reordered_levels_per_char() → reorder_visual() → invert

**Display rendering?**
→ Grid stays logical, reorder only RenderableCells
→ Move entire cells, mark lines as damaged

**Testing?**
→ See `references/testing_strategy.md`

## References

For detailed specifications and examples, see:
- `references/escape_sequences.md` - Complete escape sequence reference
- `references/uax9_algorithm.md` - Detailed UAX#9 algorithm explanation
- `references/testing_strategy.md` - Comprehensive test cases
- `references/external_links.md` - Terminal-wg spec, UAX#9, VTE implementation

**Remember**: Terminal BiDi is display-only. The grid stays in logical order, always.

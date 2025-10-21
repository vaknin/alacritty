---
name: bidi-expert
description: Expert guidance for implementing Right-to-Left (RTL) and Bidirectional (BiDi) text support in Alacritty terminal emulator. Covers terminal-wg specification, Unicode BiDi Algorithm (UAX#9), escape sequences (CSI 8, SCP, 2500, 2501, 1243), Hebrew/Arabic text rendering, logical-to-visual reordering, and BiDi implementation patterns. Use when working on BiDi features, RTL text handling, or debugging character positioning issues.
---

# BiDi Expert for Alacritty

Expert guidance for implementing Right-to-Left (RTL) and Bidirectional text support in Alacritty according to terminal-wg specification and Unicode BiDi Algorithm (UAX#9).

## Current Implementation Status

**Phase 5 - Code Refinement**
- ✅ Pure RTL text works correctly (Hebrew, Arabic)
- ✅ Character reordering follows UAX#9
- ✅ All escape sequences recognized
- ⚠️ Mixed RTL/LTR bug: LTR letters between RTL letters may appear wrong
- 🎯 Goal: Fix mixed text using BiDi algorithm's level information

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

## Critical Bugs Fixed

### Bug 1: Wrong Paragraph Direction
**Issue**: RTL text appeared right-aligned instead of left-aligned
**Fix**: Default to LTR (level 0), not RTL
**Lesson**: Terminals are left-aligned by default

### Bug 2: Display Rendering Glitches
**Issue**: Wrong characters shown temporarily
**Fix**: Reversed iteration + per-line damage tracking
**Lesson**: Manual damage marking required for BiDi

### Bug 3: Color Preservation
**Issue**: Characters appeared faded after reordering
**Fix**: Move entire cells, not just characters
**Lesson**: Never separate character from its properties

### Bug 4: Mixed RTL/LTR Positioning (Current)
**Issue**: LTR letters between RTL letters appear wrong
**Fix**: Use canonical `reorder_visual()` mapping
**Status**: Implementation complete, needs testing

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

## Testing Quick Reference

**Pure LTR**: "hello" → identity mapping [0,1,2,3,4]
**Pure RTL**: "שלום" → full reversal [3,2,1,0], visual "םולש"
**Mixed**: "hello שלום world" → three runs, middle reversed
**UTF-8**: Hebrew/Arabic (3 bytes/char) → byte_to_char mapping
**Trailing**: "שלום    " → only "שלום" processed

## References

For detailed specifications and examples, see:
- `references/escape_sequences.md` - Complete escape sequence reference
- `references/uax9_algorithm.md` - Detailed UAX#9 algorithm explanation
- `references/testing_strategy.md` - Comprehensive test cases
- `references/external_links.md` - Terminal-wg spec, UAX#9, VTE implementation

**Remember**: Terminal BiDi is display-only. The grid stays in logical order, always.

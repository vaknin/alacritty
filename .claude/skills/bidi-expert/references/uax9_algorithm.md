# Unicode BiDi Algorithm (UAX#9) Reference

Detailed explanation of UAX#9 as implemented in terminals using the `unicode-bidi` crate.

## Overview

The Unicode Bidirectional Algorithm (UAX#9) defines how to display text containing both LTR and RTL characters. The full spec has 13 rules across 6 phases. This reference covers what matters for terminal implementation.

**Key Insight**: Terminals use **implicit mode** - no explicit directional formatting characters (RLE, LRE, PDF, etc.). We rely on the algorithm's automatic level assignment.

## Character Types

UAX#9 classifies each Unicode character into types:

### Strong Types
- **L** (Left-to-Right): Latin, Cyrillic, Greek, etc.
- **R** (Right-to-Left): Hebrew
- **AL** (Arabic Letter): Arabic, Thaana, N'Ko

Strong characters have inherent directionality.

### Weak Types
- **EN** (European Number): ASCII digits 0-9
- **AN** (Arabic Number): Arabic-Indic digits
- **ES** (European Separator): +, -
- **ET** (European Terminator): $, %, #
- **CS** (Common Separator): comma, colon, slash
- **NSM** (Non-Spacing Mark): Combining diacritics
- **BN** (Boundary Neutral): Default ignorables

Weak characters take direction from surrounding context.

### Neutral Types
- **B** (Paragraph Separator): Newline, CR, etc.
- **S** (Segment Separator): Tab
- **WS** (Whitespace): Space, NBSP
- **ON** (Other Neutral): Punctuation, symbols, emoji

Neutral characters are resolved based on embedding levels.

## Algorithm Phases

### Phase 1: Paragraph Level (P1-P3)

**P1**: Split text into paragraphs (separated by type B characters)

For terminals: Usually one paragraph per line.

**P2-P3**: Determine base paragraph embedding level

```
If explicit level specified:
    base_level = explicit_level
Else if auto-detect enabled:
    Find first strong character (L, R, or AL)
    If first strong is R or AL:
        base_level = 1 (RTL)
    Else:
        base_level = 0 (LTR)
Else:
    base_level = 0 (LTR default for terminals)
```

**Terminal Implementation**:
```rust
let para_level = if use_auto_detect {
    None  // unicode-bidi crate will auto-detect
} else {
    Some(Level::ltr())  // Terminal default
};

let bidi_info = BidiInfo::new(&text, para_level);
```

### Phase 2: Explicit Levels (X1-X10)

Handles explicit directional formatting characters (RLE, LRE, RLO, LRO, PDF, FSI, PDI).

**Terminal Implementation**: Not applicable - implicit mode doesn't use these characters.

If encountered, they're treated as neutral characters by the algorithm.

### Phase 3: Weak Types (W1-W7)

Resolves weak character types based on surrounding strong types.

**Key Rules**:
- **W2**: European numbers (EN) between Arabic letters (AL) become Arabic numbers (AN)
- **W5**: European terminators (ET) adjacent to numbers become numbers
- **W7**: European numbers (EN) take direction of preceding strong type

**Example**: "hello 123 שלום"
- Base level: 0 (LTR)
- "hello " → L (level 0)
- "123" → EN, resolved to level 0 (follows preceding L)
- " " → WS (neutral)
- "שלום" → R (level 1)

### Phase 4: Neutral Types (N0-N2)

Resolves neutral characters based on embedding levels of surrounding text.

**Key Rules**:
- **N1**: Neutrals between same-direction characters take that direction
- **N2**: Remaining neutrals take paragraph embedding level

**Example**: "hello - שלום"
- "hello " → level 0
- "-" → ON (neutral between L and R)
- " " → WS
- "שלום" → level 1

The "-" is resolved: between LTR and RTL, takes paragraph level (0).

### Phase 5: Implicit Levels (I1-I2)

Assigns final embedding levels to characters.

**I1**: For characters at even level (LTR):
- R → level + 1
- AN, EN → level + 2

**I2**: For characters at odd level (RTL):
- L, EN, AN → level + 1

**Result**: Each character has a final embedding level determining its direction.

### Phase 6: Reordering (L1-L4)

The display transformation phase - what we actually implement!

#### L1: Reset Levels

Reset levels for segment/paragraph separators and trailing whitespace:

```rust
// Terminal implementation: Trim trailing cells before algorithm
while content_len > 0 && cells[content_len - 1].c == ' ' {
    content_len -= 1;
}
```

#### L2: Reorder from Highest to Lowest Level

**Core Reordering Algorithm**:

1. Find highest embedding level in text
2. Reverse all runs at that level
3. Find next-highest level
4. Reverse all runs at that level
5. Repeat until reaching base level

**Example**: "hello שלום world"
- Levels: [0,0,0,0,0,0, 1,1,1,1, 0,0,0,0,0]
- Highest level: 1
- Reverse level 1 run: "hello םולש world"

#### L3: Whitespace Handling

Whitespace at line boundaries takes the paragraph level.

#### L4: Character Mirroring

Mirror characters that have paired forms when displayed in RTL:
- Brackets: ( ) [ ] { }
- Comparison: < >
- Math: ≤ ≥

**Terminal Extension**: Box-drawing characters (if CSI ? 2500 h enabled)

## Implementation Using unicode-bidi Crate

### Basic Usage

```rust
use unicode_bidi::{BidiInfo, BidiClass, Level};

// 1. Create BidiInfo
let text = "hello שלום world";
let para_level = Some(Level::ltr());
let bidi_info = BidiInfo::new(text, para_level);

// 2. Get paragraph info
let para = &bidi_info.paragraphs[0];

// 3. Get visual runs (already reordered!)
let (levels, runs) = bidi_info.visual_runs(para, para.range.clone());

// 4. Process each run
for run in runs.iter() {
    if levels[run.start].is_rtl() {
        // RTL run: reverse characters
        process_reversed(&text[run.clone()]);
    } else {
        // LTR run: keep order
        process_normal(&text[run.clone()]);
    }
}
```

### Position Mapping (Canonical Algorithm)

For cursor movement and visual↔logical conversion:

```rust
// Get reordered levels for each character
let reordered_levels = bidi_info.reordered_levels_per_char(para, para.range.clone());

// Get visual→logical mapping (canonical from UAX#9)
let visual_to_logical = BidiInfo::reorder_visual(&reordered_levels);

// Invert to get logical→visual
let mut logical_to_visual = vec![0; text.len()];
for (visual_pos, &logical_pos) in visual_to_logical.iter().enumerate() {
    logical_to_visual[logical_pos] = visual_pos;
}
```

**Critical**: Always use `reorder_visual()` for position mapping. Character matching is fragile and fails with duplicates.

### Multi-byte UTF-8 Handling

The `unicode-bidi` crate works on byte indices, but terminals work on character indices:

```rust
// Build byte→char mapping
let mut byte_to_char = vec![0; text.len()];
for (char_idx, (byte_idx, ch)) in text.char_indices().enumerate() {
    for i in 0..ch.len_utf8() {
        byte_to_char[byte_idx + i] = char_idx;
    }
}

// Use mapping to convert run byte indices to char indices
for run in runs.iter() {
    let char_start = byte_to_char[run.start];
    let char_end = byte_to_char.get(run.end).copied().unwrap_or(num_cols);
    // Now use char indices to index into cell array
}
```

## Examples with Levels

### Example 1: Pure LTR

Text: "hello"

```
Characters:  h  e  l  l  o
Levels:      0  0  0  0  0
Visual:      h  e  l  l  o  (unchanged)
Mapping:     0→0, 1→1, 2→2, 3→3, 4→4
```

### Example 2: Pure RTL

Text: "שלום" (shin, lamed, vav, mem)

```
Characters:  ש  ל  ו  ם
Levels:      1  1  1  1
Visual:      ם  ו  ל  ש  (reversed)
Mapping:     0→3, 1→2, 2→1, 3→0
```

### Example 3: Mixed RTL/LTR

Text: "hello שלום world"

```
Characters:  h e l l o   ש ל ו ם   w o r l d
Levels:      0 0 0 0 0 0 1 1 1 1 0 0 0 0 0 0
Runs:        [----LTR---] [--RTL-] [---LTR---]
Visual:      h e l l o   ם ו ל ש   w o r l d
```

Only the RTL run gets reversed.

### Example 4: Numbers in Mixed Text

Text: "hello 123 שלום 456"

```
Characters:  h e l l o   1 2 3   ש ל ו ם   4 5 6
Types:       L L L L L W E E E W R R R R W E E E
Levels:      0 0 0 0 0 0 0 0 0 0 1 1 1 1 0 0 0 0
Visual:      h e l l o   1 2 3   ם ו ל ש   4 5 6
```

Numbers (EN) keep LTR direction due to W7 rule.

### Example 5: RTL with Punctuation

Text: "שלום!"

```
Characters:  ש ל ו ם !
Types:       R R R R ON
Levels:      1 1 1 1 1
Visual:      ! ם ו ל ש
```

Punctuation (ON) takes direction from surrounding RTL text.

## Performance Considerations

### When to Run Algorithm

Only run UAX#9 when needed:

```rust
fn line_needs_bidi(cells: &[Cell]) -> bool {
    cells.iter().any(|cell| {
        is_rtl_char(cell.c) || !cell.bidi_flags().is_empty()
    })
}
```

### RTL Character Quick Check

```rust
fn is_rtl_char(c: char) -> bool {
    matches!(c,
        '\u{0590}'..='\u{08FF}' |  // Hebrew, Arabic, Syriac, Thaana
        '\u{FB1D}'..='\u{FDFF}' |  // Hebrew/Arabic presentation forms
        '\u{FE70}'..='\u{FEFF}' |  // Arabic presentation forms
        // ... other RTL blocks
    )
}
```

### Caching

The `unicode-bidi` crate is designed for efficiency:
- Character type lookup is O(1) using tables
- Level assignment is O(n) single pass
- Reordering is O(n log n) worst case

For typical terminal usage (single-line reordering), performance is excellent.

## External Resources

- **UAX#9 Full Spec**: https://www.unicode.org/reports/tr9/
- **unicode-bidi crate**: https://docs.rs/unicode-bidi/
- **Character Database**: https://www.unicode.org/Public/UCD/latest/

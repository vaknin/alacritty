# BiDi Testing Strategy

Comprehensive test cases for BiDi implementation validation.

## Testing Principles

### 1. Test Data Correctness

**Always use real RTL text**, not Latin approximations:
- ✅ Hebrew: "שלום" (shalom)
- ✅ Arabic: "مرحبا" (marhaba)
- ❌ Fake RTL: "OLLEH" (reversed "HELLO")

### 2. Visual Verification

**What to verify**:
- Visual appearance on screen
- Logical grid contents (unchanged)
- Position mapping accuracy
- Cell properties preservation (colors, flags)

### 3. Edge Cases

**Always test**:
- Empty strings
- Single characters
- Trailing whitespace
- Multi-byte UTF-8 characters
- Mixed scripts
- Numbers and punctuation

## Core Test Cases

### Test 1: Pure LTR (Identity Mapping)

**Input**: "hello"

**Expected**:
- Visual order: "hello"
- Logical order: "hello" (unchanged)
- Mapping: [0→0, 1→1, 2→2, 3→3, 4→4]
- Levels: [0, 0, 0, 0, 0]

**Verification**:
```rust
assert_eq!(reordered[0].c, 'h');
assert_eq!(logical_to_visual, vec![0, 1, 2, 3, 4]);
```

**Why Important**: Ensures BiDi doesn't break normal LTR text.

### Test 2: Pure RTL (Full Reversal)

**Input**: "שלום" (shin ש, lamed ל, vav ו, mem ם)

**Expected**:
- Visual order: "םולש" (reversed)
- Logical order: "שלום" (unchanged)
- Mapping: [0→3, 1→2, 2→1, 3→0]
- Levels: [1, 1, 1, 1]

**Verification**:
```rust
assert_eq!(reordered[0].c, 'ם');  // First visual char is final mem
assert_eq!(reordered[3].c, 'ש');  // Last visual char is shin
assert_eq!(logical_to_visual, vec![3, 2, 1, 0]);
```

**Why Important**: Core RTL functionality.

### Test 3: Mixed RTL/LTR (Run Separation)

**Input**: "hello שלום world"

**Expected**:
- Visual order: "hello םולש world"
- Runs: 3 runs (LTR, RTL, LTR)
  - Run 1 [0-5]: "hello " (level 0, unchanged)
  - Run 2 [6-9]: "םולש" (level 1, reversed)
  - Run 3 [10-14]: "world" (level 0, unchanged)
- Levels: [0,0,0,0,0,0, 1,1,1,1, 0,0,0,0,0]

**Verification**:
```rust
// First run unchanged
assert_eq!(reordered[0..6].iter().map(|c| c.c).collect::<String>(), "hello ");

// Middle run reversed
assert_eq!(reordered[6..10].iter().map(|c| c.c).collect::<String>(), "םולש");

// Last run unchanged
assert_eq!(reordered[10..15].iter().map(|c| c.c).collect::<String>(), "world");
```

**Why Important**: Tests run boundary detection, the most complex case.

### Test 4: Multi-byte UTF-8

**Input**: "שָׁלוֹם" (Hebrew with diacritics: shin with dot, lamed, vav with dot, mem)

**Characters** (byte lengths):
- ש (3 bytes) + combining dot (3 bytes)
- ל (2 bytes)
- ו (2 bytes) + combining dot (3 bytes)
- ם (2 bytes)

**Expected**:
- Byte-to-char mapping correctly handles multi-byte characters
- No off-by-one errors in run boundaries
- Combining marks stay with base characters

**Verification**:
```rust
// Build byte_to_char mapping
let mut byte_to_char = vec![0; text.len()];
for (char_idx, (byte_idx, ch)) in text.char_indices().enumerate() {
    for i in 0..ch.len_utf8() {
        byte_to_char[byte_idx + i] = char_idx;
    }
}

// Verify all bytes of multi-byte char map to same index
assert_eq!(byte_to_char[0], byte_to_char[2]);  // All 3 bytes of ש
```

**Why Important**: UTF-8 is variable-width; incorrect byte handling causes corruption.

### Test 5: Trailing Whitespace

**Input**: "שלום    " (with 4 trailing spaces)

**Expected**:
- Content length: 4 (trailing spaces ignored)
- Only "שלום" processed by BiDi algorithm
- Visual: "םולש" (reversed, no trailing spaces)

**Verification**:
```rust
let content_len = trim_trailing(cells);
assert_eq!(content_len, 4);  // Only Hebrew chars
assert_eq!(reordered.len(), 4);  // No spaces in output
```

**Why Important**: Per terminal-wg spec, trailing cells should be ignored.

### Test 6: Numbers in Mixed Text

**Input**: "hello 123 שלום 456"

**Expected**:
- Numbers remain LTR (Western Arabic numerals default to LTR)
- Visual: "hello 123 םולש 456"
- Number runs keep their direction per W7 rule

**Verification**:
```rust
// Numbers stay LTR
assert_eq!(reordered[6..9].iter().map(|c| c.c).collect::<String>(), "123");
assert_eq!(reordered[14..17].iter().map(|c| c.c).collect::<String>(), "456");
```

**Why Important**: Number handling is complex in UAX#9 (W7 rule).

### Test 7: Punctuation

**Input**: "שלום!"

**Expected**:
- Punctuation is neutral, takes direction from context
- Visual: "!םולש" (punctuation moves with RTL text)

**Verification**:
```rust
assert_eq!(reordered[0].c, '!');  // Punctuation appears first visually
```

**Why Important**: Tests neutral character resolution (N1 rule).

### Test 8: Empty Line

**Input**: "" (empty string)

**Expected**:
- No BiDi processing needed
- `line_needs_bidi()` returns false
- No reordering performed

**Verification**:
```rust
assert!(!line_needs_bidi(&[]));
```

**Why Important**: Avoid unnecessary processing.

### Test 9: Single RTL Character

**Input**: "ש"

**Expected**:
- Visual: "ש" (appears unchanged, but technically reversed from itself)
- No crashes or edge cases

**Verification**:
```rust
assert_eq!(reordered.len(), 1);
assert_eq!(reordered[0].c, 'ש');
```

**Why Important**: Edge case for run detection.

### Test 10: Arabic with Shaping

**Input**: "مرحبا" (Arabic letters that require shaping)

**Note**: BiDi is separate from shaping. BiDi handles reordering, shaping handles glyph forms.

**Expected**:
- Characters reversed: "ابحرم"
- Shaping happens separately (not part of BiDi)

**Verification**:
```rust
// Verify logical order unchanged
assert_eq!(grid[line][0].c, 'م');  // First logical char

// Verify visual order reversed
assert_eq!(reordered[0].c, 'ا');  // First visual char
```

**Why Important**: Clarifies BiDi vs shaping responsibilities.

## Cell Properties Tests

### Test 11: Color Preservation

**Input**: "hello שלום world" with different colors per word

**Expected**:
- Each character keeps its original color
- Colors move with characters during reordering
- No "darkening" or "fading" artifacts

**Verification**:
```rust
// Set colors
grid[line][0].fg = Color::RED;    // 'h' is red
grid[line][6].fg = Color::BLUE;   // 'ש' is blue

// After reordering
assert_eq!(reordered[0].fg, Color::RED);   // 'h' still red at visual pos 0
assert_eq!(reordered[6].fg, Color::BLUE);  // 'ם' (from pos 9) is blue at pos 6
```

**Why Important**: Bug #3 was color not moving with character.

### Test 12: Hyperlink Preservation

**Input**: RTL text with hyperlinks

**Expected**:
- Hyperlink IDs move with characters
- Click targets remain correct after reordering

**Verification**:
```rust
// Set hyperlink
grid[line][6].hyperlink = Some(HyperlinkId(1));

// After reordering, hyperlink moves to visual position
assert_eq!(reordered[6].hyperlink, Some(HyperlinkId(1)));
```

### Test 13: Flags Preservation

**Input**: RTL text with various cell flags (hidden, wide, etc.)

**Expected**:
- All flags move with their characters

**Verification**:
```rust
grid[line][6].flags.insert(Flags::WIDE_CHAR);

assert!(reordered[visual_pos].flags.contains(Flags::WIDE_CHAR));
```

## Position Mapping Tests

### Test 14: Logical→Visual Cursor

**Input**: "hello שלום" with cursor at logical position 6 (first Hebrew char ש)

**Expected**:
- Visual position: 9 (last position after reversal)
- Mapping: logical[6] → visual[9]

**Verification**:
```rust
let logical_pos = 6;
let visual_pos = logical_to_visual[logical_pos];
assert_eq!(visual_pos, 9);
```

**Why Important**: Cursor must appear in correct visual position.

### Test 15: Visual→Logical Click

**Input**: User clicks at visual position 6 in "hello שלום"

**Expected**:
- Logical position: 9 (last Hebrew char ם)
- Mapping: visual[6] → logical[9]

**Verification**:
```rust
let visual_pos = 6;
let logical_pos = visual_to_logical[visual_pos];
assert_eq!(logical_pos, 9);
```

**Why Important**: Mouse clicks must select correct logical character.

## Escape Sequence Tests

### Test 16: BiDi Mode Enable/Disable

**Sequence**: `CSI 8 h` (enable), `CSI 8 l` (disable)

**Expected**:
- When enabled: BiDi reordering active
- When disabled: No reordering (visual = logical)

**Verification**:
```rust
// After CSI 8 l
assert!(!line_needs_bidi(&cells));
```

### Test 17: Explicit Paragraph Direction

**Sequence**: `CSI 1 SP k` (LTR), `CSI 2 SP k` (RTL)

**Input**: "שלום"

**Expected**:
- CSI 1 SP k: LTR paragraph (default terminal behavior)
- CSI 2 SP k: RTL paragraph (right-aligned, but Alacritty may not implement alignment)

**Verification**:
```rust
assert!(cell.bidi_flags().contains(BidiFlags::NON_DEFAULT_PARA_DIR));
```

### Test 18: Auto-detection

**Sequence**: `CSI ? 2501 h`

**Input 1**: "שלום hello" (RTL first)
**Expected**: RTL paragraph detected

**Input 2**: "hello שלום" (LTR first)
**Expected**: LTR paragraph detected

**Verification**:
```rust
assert!(cell.bidi_flags().contains(BidiFlags::AUTO_PARA_DIR));
```

## Performance Tests

### Test 19: Large Lines

**Input**: 1000-character line with mixed RTL/LTR

**Expected**:
- Processing completes in < 1ms
- No memory leaks
- Correct reordering throughout

### Test 20: Rapid Updates

**Input**: 100 lines updated in quick succession

**Expected**:
- No lag or dropped frames
- Damage tracking correctly marks all BiDi lines

## Integration Tests

### Test 21: Text Editors (vim, emacs)

**Test**: Open Hebrew/Arabic file, edit text

**Expected**:
- Cursor appears at correct position
- Insertion/deletion works correctly
- No visual artifacts

### Test 22: ls Command

**Test**: `ls` in directory with Hebrew filenames

**Expected**:
- Filenames display correctly reversed
- Selection works
- Colors preserved

### Test 23: cat Command

**Test**: `cat hebrew_file.txt`

**Expected**:
- Each line reordered correctly
- Scrolling works
- No corruption

## Regression Tests

### Bug #1 Regression: Paragraph Direction

**Test**: Verify RTL text appears left-aligned (not right-aligned)

**Input**: "שלום"

**Expected**: Characters start at column 0 (left side)

### Bug #2 Regression: Display Rendering

**Test**: Type characters in vim, verify no flickering

**Expected**: Stable rendering, no wrong characters shown

### Bug #3 Regression: Color Preservation

**Test**: Insert space between colored RTL characters

**Expected**: Colors move with characters, no fading

### Bug #4 Regression: Mixed RTL/LTR

**Test**: Type Latin 'a' between Hebrew 'א' and 'ב'

**Expected**: Latin character appears in correct visual position

## Test Utilities

### Visual Comparison Tool

```rust
fn assert_visual_order(input: &str, expected_visual: &str) {
    let cells = create_cells_from_string(input);
    let (reordered, _mapping) = reorder_line(&cells, 0);
    let actual_visual: String = reordered.iter().map(|c| c.c).collect();
    assert_eq!(actual_visual, expected_visual);
}
```

### Mapping Verification

```rust
fn assert_mapping(input: &str, expected_mapping: &[(usize, usize)]) {
    let cells = create_cells_from_string(input);
    let (_reordered, mapping) = reorder_line(&cells, 0);
    for (logical, visual) in expected_mapping {
        assert_eq!(mapping.logical_to_visual[*logical], *visual);
    }
}
```

## Manual Testing Checklist

- [ ] Pure LTR text unchanged
- [ ] Pure RTL text reversed
- [ ] Mixed text with correct run boundaries
- [ ] Colors preserved during reordering
- [ ] Hyperlinks work after reordering
- [ ] Cursor appears at correct visual position
- [ ] Mouse clicks select correct character
- [ ] vim/emacs editing works
- [ ] ls with RTL filenames works
- [ ] No memory leaks in long sessions
- [ ] No performance degradation

## Test Data Sources

**Hebrew Test Strings**:
- "שלום" (shalom - hello)
- "שלום עולם" (shalom olam - hello world)
- "אבג" (aleph bet gimel - ABC)

**Arabic Test Strings**:
- "مرحبا" (marhaba - hello)
- "مرحبا بالعالم" (marhaba bel 'alam - hello world)

**Mixed Test Strings**:
- "hello שלום world"
- "test 123 בדיקה 456"
- "RTL: شلום, LTR: hello"

## Automated Test Framework

Consider using:
- `cargo test` for unit tests
- `vtebench` for performance regression
- `typometer` for input latency measurement
- Custom visual diff tool for screenshot comparison

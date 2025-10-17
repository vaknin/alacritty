//! Bidirectional text support.

use unicode_bidi::{BidiInfo, Level};

use crate::grid::Row;
use crate::term::cell::{BidiDir, BidiMode, Cell};

/// Check if a line contains RTL text or BiDi control flags.
///
/// This is a fast check that determines whether we need to run the full BiDi algorithm.
#[inline]
pub fn line_needs_bidi(row: &Row<Cell>) -> bool {
    for cell in row {
        // Check for explicit BiDi flags
        if cell.bidi_flags().bits() != 0 {
            return true;
        }

        // Check for RTL Unicode characters (Hebrew, Arabic, etc.)
        let c = cell.c;
        if is_rtl_char(c) {
            return true;
        }
    }

    false
}

/// Check if a character is in an RTL Unicode block.
#[inline]
fn is_rtl_char(c: char) -> bool {
    matches!(c,
        // Hebrew: U+0590 - U+05FF
        '\u{0590}'..='\u{05FF}' |
        // Arabic: U+0600 - U+06FF
        '\u{0600}'..='\u{06FF}' |
        // Arabic Supplement: U+0750 - U+077F
        '\u{0750}'..='\u{077F}' |
        // Arabic Extended-A: U+08A0 - U+08FF
        '\u{08A0}'..='\u{08FF}' |
        // Arabic Presentation Forms-A: U+FB50 - U+FDFF
        '\u{FB50}'..='\u{FDFF}' |
        // Arabic Presentation Forms-B: U+FE70 - U+FEFF
        '\u{FE70}'..='\u{FEFF}'
    )
}

/// Reorder a line of cells according to the Unicode Bidirectional Algorithm.
///
/// Returns a tuple of:
/// - Vec of cells in visual order (same length as input row)
/// - Vec of position mappings: `logical_to_visual[logical_pos] = visual_pos`
pub fn reorder_line(row: &Row<Cell>) -> (Vec<Cell>, Vec<usize>) {
    let num_cols = row.len();

    // Collect cells into a vec.
    let cells: Vec<Cell> = row.into_iter().cloned().collect();

    // Per terminal-wg spec: "Trailing erased cells should be ignored when processing
    // the paragraph for BiDi algorithm". We trim trailing spaces/empty cells.
    let mut content_len = cells.len();
    while content_len > 0 && cells[content_len - 1].c == ' ' && cells[content_len - 1].flags.is_empty() {
        content_len -= 1;
    }

    // If the entire line is empty, return cells as-is with identity mapping.
    if content_len == 0 {
        let identity_mapping: Vec<usize> = (0..num_cols).collect();
        return (cells, identity_mapping);
    }

    // Extract text only from non-trailing cells.
    let text: String = cells[..content_len].iter().map(|cell| cell.c).collect();

    // Determine paragraph direction from cell BiDi modes (only check non-trailing content).
    // Default to LTR (left-aligned) as per terminal-wg spec and VTE behavior.
    let mut use_auto_detect = false;
    let mut explicit_level = None;

    for cell in &cells[..content_len] {
        let flags = cell.bidi_flags();
        if flags.bits() == 0 {
            continue;
        }

        match cell.bidi_mode() {
            BidiMode::Auto { .. } => {
                // Auto mode (CSI ? 2501 h) enables auto-detection.
                use_auto_detect = true;
                break; // Auto mode takes precedence.
            },
            BidiMode::Explicit { forced_dir } | BidiMode::Implicit { para_dir: forced_dir } => {
                if explicit_level.is_none() {
                    explicit_level = match forced_dir {
                        BidiDir::LTR => Some(Level::ltr()),
                        BidiDir::RTL => Some(Level::rtl()),
                        BidiDir::Default => None,
                    };
                }
            },
        }
    }

    // Determine final paragraph level:
    // - If auto-detection is enabled: use None (let unicode-bidi detect direction).
    // - If explicit level is set: use that level.
    // - Otherwise: default to LTR (terminals are left-aligned by default).
    let para_level = if use_auto_detect {
        None
    } else {
        explicit_level.or(Some(Level::ltr()))
    };

    // Run the BiDi algorithm.
    let bidi_info = BidiInfo::new(&text, para_level);

    // Get the first (and only) paragraph.
    let para = &bidi_info.paragraphs[0];

    // Get visual runs - this gives us runs in visual order with byte indices.
    // This is the proper API (like VTE uses FriBidi's run API).
    let (levels, runs) = bidi_info.visual_runs(para, para.range.clone());

    // Build byte-to-char mapping for multi-byte UTF-8 characters.
    // Each byte of a multi-byte char must map to the same character index.
    let mut byte_to_char: Vec<usize> = vec![0; text.len()];
    for (char_idx, (byte_idx, ch)) in text.char_indices().enumerate() {
        let char_len = ch.len_utf8();
        for i in 0..char_len {
            byte_to_char[byte_idx + i] = char_idx;
        }
    }

    // Reorder cells based on visual runs (only the non-trailing content).
    let mut reordered_cells: Vec<Cell> = Vec::with_capacity(num_cols);

    for run in runs.iter() {
        // Convert byte indices to character indices.
        let char_start = byte_to_char.get(run.start).copied().unwrap_or(0);
        let char_end = byte_to_char.get(run.end).copied().unwrap_or(content_len);

        if levels[run.start].is_rtl() {
            // RTL run: reverse the cells.
            reordered_cells.extend(cells[char_start..char_end].iter().rev().cloned());
        } else {
            // LTR run: keep cells in order.
            reordered_cells.extend_from_slice(&cells[char_start..char_end]);
        }
    }

    // Append trailing empty cells as-is (per terminal-wg spec: they map to the end).
    reordered_cells.extend_from_slice(&cells[content_len..]);

    // Ensure we have exactly num_cols cells (pad if needed).
    while reordered_cells.len() < num_cols {
        reordered_cells.push(Cell::default());
    }
    reordered_cells.truncate(num_cols);

    // Build logical→visual position mapping using the BiDi algorithm's canonical mapping.
    // This eliminates ambiguity when handling mixed RTL/LTR text.
    let reordered_levels = bidi_info.reordered_levels_per_char(para, para.range.clone());
    let visual_to_logical = BidiInfo::reorder_visual(&reordered_levels);

    // Invert the mapping: visual_to_logical[visual_pos] = logical_pos
    // → logical_to_visual[logical_pos] = visual_pos
    let mut logical_to_visual: Vec<usize> = vec![0; num_cols];
    for (visual_pos, &logical_pos) in visual_to_logical.iter().enumerate() {
        if logical_pos < content_len {
            logical_to_visual[logical_pos] = visual_pos;
        }
    }

    // Trailing cells map to themselves at the end.
    for pos in content_len..num_cols {
        logical_to_visual[pos] = pos;
    }

    (reordered_cells, logical_to_visual)
}

/// Convert a visual column position to logical column position for a row.
///
/// If the row doesn't need BiDi reordering, returns the column unchanged.
/// Otherwise, uses the inverse of the logical-to-visual mapping.
#[cfg(feature = "bidi_draft")]
pub fn visual_column_to_logical(row: &Row<Cell>, visual_col: usize) -> usize {
    if !line_needs_bidi(row) {
        return visual_col;
    }

    let (_reordered, logical_to_visual) = reorder_line(row);

    // Invert the mapping: find which logical position maps to this visual position.
    for (logical_pos, &visual_pos) in logical_to_visual.iter().enumerate() {
        if visual_pos == visual_col {
            return logical_pos;
        }
    }

    // Fallback if visual_col is out of range or not found.
    visual_col
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Row;
    use crate::index::Column;
    use crate::term::cell::Cell;

    #[test]
    fn test_is_rtl_char() {
        assert!(is_rtl_char('ש')); // Hebrew
        assert!(is_rtl_char('ا')); // Arabic
        assert!(!is_rtl_char('a')); // Latin
        assert!(!is_rtl_char('1')); // Digit
    }

    #[test]
    fn test_line_needs_bidi_ltr_only() {
        let mut row = Row::<Cell>::new(10);
        row[Column(0)].c = 'h';
        row[Column(1)].c = 'e';
        row[Column(2)].c = 'l';
        row[Column(3)].c = 'l';
        row[Column(4)].c = 'o';

        assert!(!line_needs_bidi(&row));
    }

    #[test]
    fn test_line_needs_bidi_with_hebrew() {
        let mut row = Row::<Cell>::new(10);
        row[Column(0)].c = 'ש';
        row[Column(1)].c = 'ל';
        row[Column(2)].c = 'ו';
        row[Column(3)].c = 'ם';

        assert!(line_needs_bidi(&row));
    }

    #[test]
    fn test_reorder_line_pure_ltr() {
        let mut row = Row::<Cell>::new(3);
        row[Column(0)].c = 'a';
        row[Column(1)].c = 'b';
        row[Column(2)].c = 'c';

        let (reordered, mapping) = reorder_line(&row);
        assert_eq!(reordered[0].c, 'a');
        assert_eq!(reordered[1].c, 'b');
        assert_eq!(reordered[2].c, 'c');
        // LTR: identity mapping
        assert_eq!(mapping[0], 0);
        assert_eq!(mapping[1], 1);
        assert_eq!(mapping[2], 2);
    }

    #[test]
    fn test_reorder_line_pure_rtl() {
        let mut row = Row::<Cell>::new(4);
        row[Column(0)].c = 'ש'; // shin
        row[Column(1)].c = 'ל'; // lamed
        row[Column(2)].c = 'ו'; // vav
        row[Column(3)].c = 'ם'; // final mem

        let (reordered, mapping) = reorder_line(&row);
        // RTL text should be reversed
        assert_eq!(reordered[0].c, 'ם');
        assert_eq!(reordered[1].c, 'ו');
        assert_eq!(reordered[2].c, 'ל');
        assert_eq!(reordered[3].c, 'ש');
        // RTL: reversed mapping (logical 0 → visual 3, etc.)
        assert_eq!(mapping[0], 3);
        assert_eq!(mapping[1], 2);
        assert_eq!(mapping[2], 1);
        assert_eq!(mapping[3], 0);
    }

    #[test]
    fn test_reorder_line_different_rtl_chars() {
        // This test specifically checks the bug where different Hebrew letters
        // were causing rendering issues due to incorrect byte-to-char mapping.
        let mut row = Row::<Cell>::new(3);
        row[Column(0)].c = 'א'; // alef
        row[Column(1)].c = 'ב'; // bet
        row[Column(2)].c = 'ג'; // gimel

        let (reordered, mapping) = reorder_line(&row);
        // RTL text should be reversed: אבג → גבא
        assert_eq!(reordered[0].c, 'ג', "First position should show gimel");
        assert_eq!(reordered[1].c, 'ב', "Second position should show bet");
        assert_eq!(reordered[2].c, 'א', "Third position should show alef");
        // Mapping should also be reversed
        assert_eq!(mapping[0], 2);
        assert_eq!(mapping[1], 1);
        assert_eq!(mapping[2], 0);
    }

    #[test]
    #[cfg(feature = "bidi_draft")]
    fn test_visual_to_logical_column_pure_ltr() {
        let mut row = Row::<Cell>::new(3);
        row[Column(0)].c = 'a';
        row[Column(1)].c = 'b';
        row[Column(2)].c = 'c';

        // LTR has identity mapping.
        assert_eq!(visual_column_to_logical(&row, 0), 0);
        assert_eq!(visual_column_to_logical(&row, 1), 1);
        assert_eq!(visual_column_to_logical(&row, 2), 2);
    }

    #[test]
    #[cfg(feature = "bidi_draft")]
    fn test_visual_to_logical_column_pure_rtl() {
        let mut row = Row::<Cell>::new(3);
        row[Column(0)].c = 'א'; // alef
        row[Column(1)].c = 'ב'; // bet
        row[Column(2)].c = 'ג'; // gimel

        // Logical: א(0) ב(1) ג(2)
        // Visual: ג(0) ב(1) א(2) [reversed for display]
        // visual_col 0 (see ג) → logical 2
        // visual_col 1 (see ב) → logical 1
        // visual_col 2 (see א) → logical 0
        assert_eq!(visual_column_to_logical(&row, 0), 2);
        assert_eq!(visual_column_to_logical(&row, 1), 1);
        assert_eq!(visual_column_to_logical(&row, 2), 0);
    }
}

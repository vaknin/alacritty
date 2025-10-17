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
/// Returns a Vec of cells in visual order.
/// The returned Vec has the same length as the input row.
pub fn reorder_line(row: &Row<Cell>) -> Vec<Cell> {
    let num_cols = row.len();

    // Collect cells into a vec.
    let cells: Vec<Cell> = row.into_iter().cloned().collect();

    // Extract text from cells.
    let text: String = cells.iter().map(|cell| cell.c).collect();

    // Determine paragraph direction from first cell with explicit BiDi mode.
    let para_level = cells
        .iter()
        .find_map(|cell| {
            if cell.bidi_flags().bits() == 0 {
                return None;
            }

            match cell.bidi_mode() {
                BidiMode::Explicit { forced_dir } | BidiMode::Implicit { para_dir: forced_dir } => {
                    match forced_dir {
                        BidiDir::LTR => Some(Level::ltr()),
                        BidiDir::RTL => Some(Level::rtl()),
                        BidiDir::Default => None,
                    }
                },
                BidiMode::Auto { .. } => None,
            }
        });

    // Run the BiDi algorithm.
    let bidi_info = BidiInfo::new(&text, para_level);

    // Get the first (and only) paragraph.
    let para = &bidi_info.paragraphs[0];

    // Get visual runs (these are in visual order, but use byte indices).
    let line_range = para.range.clone();
    let (levels, runs) = bidi_info.visual_runs(para, line_range);

    // Build a mapping from byte index to character index.
    let mut byte_to_char: Vec<usize> = Vec::with_capacity(text.len());
    for (char_idx, (byte_idx, _)) in text.char_indices().enumerate() {
        byte_to_char.resize(byte_idx + 1, char_idx);
    }
    byte_to_char.resize(text.len() + 1, cells.len());

    // Reorder cells based on visual runs.
    let mut reordered_cells: Vec<Cell> = Vec::with_capacity(num_cols);

    for run in runs.iter() {
        // Convert byte indices to character indices.
        let char_start = byte_to_char.get(run.start).copied().unwrap_or(0);
        let char_end = byte_to_char.get(run.end).copied().unwrap_or(cells.len());

        let run_cells = if levels[run.start].is_rtl() {
            // RTL run: reverse the cells.
            cells[char_start..char_end].iter().rev().cloned().collect::<Vec<_>>()
        } else {
            // LTR run: keep order.
            cells[char_start..char_end].to_vec()
        };
        reordered_cells.extend(run_cells);
    }

    // Ensure we have exactly num_cols cells (pad if needed).
    while reordered_cells.len() < num_cols {
        reordered_cells.push(Cell::default());
    }
    reordered_cells.truncate(num_cols);

    reordered_cells
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

        let reordered = reorder_line(&row);
        assert_eq!(reordered[0].c, 'a');
        assert_eq!(reordered[1].c, 'b');
        assert_eq!(reordered[2].c, 'c');
    }

    #[test]
    fn test_reorder_line_pure_rtl() {
        let mut row = Row::<Cell>::new(4);
        row[Column(0)].c = 'ש'; // shin
        row[Column(1)].c = 'ל'; // lamed
        row[Column(2)].c = 'ו'; // vav
        row[Column(3)].c = 'ם'; // final mem

        let reordered = reorder_line(&row);
        // RTL text should be reversed
        assert_eq!(reordered[0].c, 'ם');
        assert_eq!(reordered[1].c, 'ו');
        assert_eq!(reordered[2].c, 'ל');
        assert_eq!(reordered[3].c, 'ש');
    }
}

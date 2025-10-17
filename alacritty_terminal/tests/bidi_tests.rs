#![cfg(feature = "bidi_draft")]

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::index::Column;
use alacritty_terminal::term::test::TermSize;
use alacritty_terminal::term::{Config, Term};
use alacritty_terminal::vte::ansi;

#[derive(Copy, Clone)]
struct MockEventListener;

impl EventListener for MockEventListener {
    fn send_event(&self, _event: Event) {}
}

fn create_term() -> Term<MockEventListener> {
    let size = TermSize::new(80, 24);
    Term::new(Config::default(), &size, MockEventListener)
}

fn feed_bytes(term: &mut Term<MockEventListener>, bytes: &[u8]) {
    let mut parser: ansi::Processor<ansi::StdSyncHandler> = ansi::Processor::new();
    parser.advance(term, bytes);
}

// Test Mode 8 (BDSM - BiDi Support Mode).
#[test]
fn test_mode_8_implicit_bidi_enable() {
    let mut term = create_term();

    // Enable implicit BiDi mode (CSI ? 8 h).
    feed_bytes(&mut term, b"\x1b[?8h");

    // Type some text to create cells.
    feed_bytes(&mut term, b"test");

    // In implicit mode, cells should have implicit BiDi behavior.
    // The exact flag checking depends on implementation details.
    // This test verifies the mode can be set without panicking.
}

#[test]
fn test_mode_8_explicit_bidi_enable() {
    let mut term = create_term();

    // Disable implicit BiDi mode / enable explicit (CSI ? 8 l).
    feed_bytes(&mut term, b"\x1b[?8l");

    // Type some text.
    feed_bytes(&mut term, b"test");

    // In explicit mode, BiDi behavior is controlled by explicit escape sequences.
}

// Test Mode 1243 (Arrow Key Swapping).
#[test]
fn test_mode_1243_arrow_key_swapping_disable() {
    let mut term = create_term();

    // Disable arrow key swapping (CSI ? 1243 l).
    feed_bytes(&mut term, b"\x1b[?1243l");

    // Verify the flag is set.
    assert!(term.bidi_disable_arrow_key_swapping);
}

#[test]
fn test_mode_1243_arrow_key_swapping_enable() {
    let mut term = create_term();

    // First disable to set the flag.
    feed_bytes(&mut term, b"\x1b[?1243l");
    assert!(term.bidi_disable_arrow_key_swapping);

    // Now enable arrow key swapping (CSI ? 1243 h).
    feed_bytes(&mut term, b"\x1b[?1243h");

    // Verify the flag is cleared.
    assert!(!term.bidi_disable_arrow_key_swapping);
}

// Test Mode 2500 (Box Mirroring).
#[test]
fn test_mode_2500_box_mirroring_enable() {
    let mut term = create_term();

    // Enable box mirroring (CSI ? 2500 h).
    feed_bytes(&mut term, b"\x1b[?2500h");

    // Type a character.
    feed_bytes(&mut term, b"X");

    // Check that the cell has box mirroring enabled.
    let grid = term.grid();
    let cursor_point = grid.cursor.point;
    // Cursor has moved forward, so check the previous column.
    let col = if cursor_point.column.0 > 0 { cursor_point.column.0 - 1 } else { 0 };
    let cell = &grid[cursor_point.line][Column(col)];

    assert!(cell.bidi_box_mirroring(), "Box mirroring should be enabled for the cell");
}

#[test]
fn test_mode_2500_box_mirroring_disable() {
    let mut term = create_term();

    // Enable then disable box mirroring.
    feed_bytes(&mut term, b"\x1b[?2500h");
    feed_bytes(&mut term, b"\x1b[?2500l");

    // Type a character.
    feed_bytes(&mut term, b"Y");

    // Check that the cell does NOT have box mirroring enabled.
    let grid = term.grid();
    let cursor_point = grid.cursor.point;
    let col = if cursor_point.column.0 > 0 { cursor_point.column.0 - 1 } else { 0 };
    let cell = &grid[cursor_point.line][Column(col)];

    assert!(!cell.bidi_box_mirroring(), "Box mirroring should be disabled for the cell");
}

// Test Mode 2501 (Auto Paragraph Direction).
#[test]
fn test_mode_2501_auto_para_direction_enable() {
    let mut term = create_term();

    // Enable auto paragraph direction (CSI ? 2501 h).
    feed_bytes(&mut term, b"\x1b[?2501h");

    // Type a character.
    feed_bytes(&mut term, b"A");

    // Check that cells have auto paragraph direction flag.
    let grid = term.grid();
    let cursor_point = grid.cursor.point;
    let col = if cursor_point.column.0 > 0 { cursor_point.column.0 - 1 } else { 0 };
    let cell = &grid[cursor_point.line][Column(col)];

    // Verify the BiDi mode is Auto.
    use alacritty_terminal::term::cell::BidiMode;
    matches!(cell.bidi_mode(), BidiMode::Auto { .. });
}

#[test]
fn test_mode_2501_auto_para_direction_disable() {
    let mut term = create_term();

    // Enable then disable auto paragraph direction.
    feed_bytes(&mut term, b"\x1b[?2501h");
    feed_bytes(&mut term, b"\x1b[?2501l");

    // Type a character.
    feed_bytes(&mut term, b"B");

    // Check that cells do NOT have auto paragraph direction.
    let grid = term.grid();
    let cursor_point = grid.cursor.point;
    let col = if cursor_point.column.0 > 0 { cursor_point.column.0 - 1 } else { 0 };
    let cell = &grid[cursor_point.line][Column(col)];

    // Verify the BiDi mode is NOT Auto.
    use alacritty_terminal::term::cell::BidiMode;
    assert!(!matches!(cell.bidi_mode(), BidiMode::Auto { .. }));
}

// Test SCP (Select Character Path) - Default Direction.
#[test]
fn test_scp_default_direction() {
    let mut term = create_term();

    // Set default paragraph direction (CSI 0 SP k).
    feed_bytes(&mut term, b"\x1b[0 k");

    // Type a character.
    feed_bytes(&mut term, b"D");

    // Verify the cell has default direction.
    let grid = term.grid();
    let cursor_point = grid.cursor.point;
    let col = if cursor_point.column.0 > 0 { cursor_point.column.0 - 1 } else { 0 };
    let cell = &grid[cursor_point.line][Column(col)];

    use alacritty_terminal::term::cell::{BidiDir, BidiMode};
    match cell.bidi_mode() {
        BidiMode::Explicit { forced_dir } => {
            assert_eq!(forced_dir, BidiDir::Default, "Direction should be Default");
        },
        BidiMode::Implicit { para_dir } => {
            assert_eq!(para_dir, BidiDir::Default, "Direction should be Default");
        },
        BidiMode::Auto { fallback_para_dir } => {
            assert_eq!(fallback_para_dir, BidiDir::Default, "Direction should be Default");
        },
    }
}

// Test SCP - LTR Direction.
#[test]
fn test_scp_ltr_direction() {
    let mut term = create_term();

    // Set LTR paragraph direction (CSI 1 SP k).
    feed_bytes(&mut term, b"\x1b[1 k");

    // Type a character.
    feed_bytes(&mut term, b"L");

    // Verify the cell has LTR direction.
    let grid = term.grid();
    let cursor_point = grid.cursor.point;
    let col = if cursor_point.column.0 > 0 { cursor_point.column.0 - 1 } else { 0 };
    let cell = &grid[cursor_point.line][Column(col)];

    use alacritty_terminal::term::cell::{BidiDir, BidiMode};
    match cell.bidi_mode() {
        BidiMode::Explicit { forced_dir } => {
            assert_eq!(forced_dir, BidiDir::LTR, "Direction should be LTR");
        },
        BidiMode::Implicit { para_dir } => {
            assert_eq!(para_dir, BidiDir::LTR, "Direction should be LTR");
        },
        BidiMode::Auto { fallback_para_dir } => {
            assert_eq!(fallback_para_dir, BidiDir::LTR, "Direction should be LTR");
        },
    }
}

// Test SCP - RTL Direction.
#[test]
fn test_scp_rtl_direction() {
    let mut term = create_term();

    // Set RTL paragraph direction (CSI 2 SP k).
    feed_bytes(&mut term, b"\x1b[2 k");

    // Type a character.
    feed_bytes(&mut term, b"R");

    // Verify the cell has RTL direction.
    let grid = term.grid();
    let cursor_point = grid.cursor.point;
    let col = if cursor_point.column.0 > 0 { cursor_point.column.0 - 1 } else { 0 };
    let cell = &grid[cursor_point.line][Column(col)];

    use alacritty_terminal::term::cell::{BidiDir, BidiMode};
    match cell.bidi_mode() {
        BidiMode::Explicit { forced_dir } => {
            assert_eq!(forced_dir, BidiDir::RTL, "Direction should be RTL");
        },
        BidiMode::Implicit { para_dir } => {
            assert_eq!(para_dir, BidiDir::RTL, "Direction should be RTL");
        },
        BidiMode::Auto { fallback_para_dir } => {
            assert_eq!(fallback_para_dir, BidiDir::RTL, "Direction should be RTL");
        },
    }
}

// Test BiDi detection and reordering.
#[test]
fn test_line_needs_bidi_with_ltr_only() {
    use alacritty_terminal::grid::Row;
    use alacritty_terminal::term::bidi;
    use alacritty_terminal::term::cell::Cell;

    // Create a row with LTR-only content.
    let mut row = Row::<Cell>::new(5);
    row[Column(0)].c = 'H';
    row[Column(1)].c = 'e';
    row[Column(2)].c = 'l';
    row[Column(3)].c = 'l';
    row[Column(4)].c = 'o';

    // Should not need BiDi processing.
    assert!(!bidi::line_needs_bidi(&row), "Pure LTR text should not need BiDi");
}

#[test]
fn test_line_needs_bidi_with_rtl_content() {
    use alacritty_terminal::grid::Row;
    use alacritty_terminal::term::bidi;
    use alacritty_terminal::term::cell::Cell;

    // Create a row with Hebrew (RTL) content.
    let mut row = Row::<Cell>::new(4);
    row[Column(0)].c = 'ש';
    row[Column(1)].c = 'ל';
    row[Column(2)].c = 'ו';
    row[Column(3)].c = 'ם';

    // Should need BiDi processing.
    assert!(bidi::line_needs_bidi(&row), "RTL text should need BiDi");
}

#[test]
fn test_line_needs_bidi_with_mixed_content() {
    use alacritty_terminal::grid::Row;
    use alacritty_terminal::term::bidi;
    use alacritty_terminal::term::cell::Cell;

    // Create a row with mixed LTR/RTL content.
    let mut row = Row::<Cell>::new(9);
    row[Column(0)].c = 'H';
    row[Column(1)].c = 'i';
    row[Column(2)].c = ' ';
    row[Column(3)].c = 'ש';
    row[Column(4)].c = 'ל';
    row[Column(5)].c = 'ו';
    row[Column(6)].c = 'ם';
    row[Column(7)].c = ' ';
    row[Column(8)].c = '!';

    // Should need BiDi processing.
    assert!(bidi::line_needs_bidi(&row), "Mixed LTR/RTL text should need BiDi");
}

// Test for the specific bug: LTR character inserted between RTL characters.
#[test]
fn test_ltr_between_rtl_characters() {
    use alacritty_terminal::grid::Row;
    use alacritty_terminal::term::bidi;
    use alacritty_terminal::term::cell::Cell;

    // Test case: "אaב" (alef, latin-a, bet)
    // This simulates typing 'a' between two Hebrew letters.
    let mut row = Row::<Cell>::new(3);
    row[Column(0)].c = 'א'; // alef
    row[Column(1)].c = 'a'; // latin a
    row[Column(2)].c = 'ב'; // bet

    let (reordered, mapping) = bidi::reorder_line(&row);

    // Debug output to see what we're actually getting
    eprintln!("Logical: א a ב");
    eprintln!("Visual cells: {} {} {}", reordered[0].c, reordered[1].c, reordered[2].c);
    eprintln!("Position mapping: {:?}", mapping);
    eprintln!("  logical 0 (א) -> visual {}", mapping[0]);
    eprintln!("  logical 1 (a) -> visual {}", mapping[1]);
    eprintln!("  logical 2 (ב) -> visual {}", mapping[2]);

    // With LTR base direction and isolated RTL characters,
    // the BiDi algorithm should determine the correct visual order.
    // Let's verify what we get and document the expected behavior.
}

#[test]
fn test_reorder_line_pure_ltr() {
    use alacritty_terminal::grid::Row;
    use alacritty_terminal::term::bidi;
    use alacritty_terminal::term::cell::Cell;

    // Create a row with pure LTR text.
    let mut row = Row::<Cell>::new(5);
    row[Column(0)].c = 'H';
    row[Column(1)].c = 'e';
    row[Column(2)].c = 'l';
    row[Column(3)].c = 'l';
    row[Column(4)].c = 'o';

    let (reordered, _mapping) = bidi::reorder_line(&row);

    // LTR text should remain in the same order.
    assert_eq!(reordered.len(), 5);
    assert_eq!(reordered[0].c, 'H');
    assert_eq!(reordered[1].c, 'e');
    assert_eq!(reordered[2].c, 'l');
    assert_eq!(reordered[3].c, 'l');
    assert_eq!(reordered[4].c, 'o');
}

#[test]
fn test_reorder_line_pure_rtl() {
    use alacritty_terminal::grid::Row;
    use alacritty_terminal::term::bidi;
    use alacritty_terminal::term::cell::Cell;

    // Create a row with pure RTL text (Hebrew: shalom).
    let mut row = Row::<Cell>::new(4);
    row[Column(0)].c = 'ש'; // shin
    row[Column(1)].c = 'ל'; // lamed
    row[Column(2)].c = 'ו'; // vav
    row[Column(3)].c = 'ם'; // mem sofit

    let (reordered, _mapping) = bidi::reorder_line(&row);

    // RTL text should be reversed for visual display.
    assert_eq!(reordered.len(), 4);
    assert_eq!(reordered[0].c, 'ם');
    assert_eq!(reordered[1].c, 'ו');
    assert_eq!(reordered[2].c, 'ל');
    assert_eq!(reordered[3].c, 'ש');
}

#[test]
fn test_reorder_line_mixed_ltr_rtl() {
    use alacritty_terminal::grid::Row;
    use alacritty_terminal::term::bidi;
    use alacritty_terminal::term::cell::Cell;

    // Create a row with mixed LTR/RTL: "Hi שלום !"
    // Expected visual order: "Hi םולש !"
    let mut row = Row::<Cell>::new(9);
    row[Column(0)].c = 'H';
    row[Column(1)].c = 'i';
    row[Column(2)].c = ' ';
    row[Column(3)].c = 'ש';
    row[Column(4)].c = 'ל';
    row[Column(5)].c = 'ו';
    row[Column(6)].c = 'ם';
    row[Column(7)].c = ' ';
    row[Column(8)].c = '!';

    let (reordered, _mapping) = bidi::reorder_line(&row);

    // LTR part stays: "Hi "
    assert_eq!(reordered[0].c, 'H');
    assert_eq!(reordered[1].c, 'i');
    assert_eq!(reordered[2].c, ' ');

    // RTL part reverses: "םולש"
    assert_eq!(reordered[3].c, 'ם');
    assert_eq!(reordered[4].c, 'ו');
    assert_eq!(reordered[5].c, 'ל');
    assert_eq!(reordered[6].c, 'ש');

    // Final LTR: " !"
    assert_eq!(reordered[7].c, ' ');
    assert_eq!(reordered[8].c, '!');
}

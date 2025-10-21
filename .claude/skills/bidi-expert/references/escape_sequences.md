# BiDi Escape Sequences Reference

Complete reference for all BiDi-related escape sequences per terminal-wg specification.

## CSI 8 h / CSI 8 l

**BiDi Implicit Mode** (BDSM - BiDi Support Mode)

- `CSI 8 h` or `ESC [ 8 h`: Enable implicit BiDi mode
- `CSI 8 l` or `ESC [ 8 l`: Disable implicit BiDi mode

**Behavior**: When enabled, terminal performs BiDi reordering automatically. This is **Level 1 Implicit Mode** from terminal-wg spec.

**Implementation**: Sets BidiMode to Implicit on affected cells.

**Usage**: Applications should enable this mode before sending RTL/BiDi text.

## CSI [0-2] SP k

**Set Character Path (SCP)** - Paragraph direction control

- `CSI 0 SP k` or `ESC [ 0   k`: Default (context-dependent, usually LTR)
- `CSI 1 SP k` or `ESC [ 1   k`: LTR (left-to-right paragraph)
- `CSI 2 SP k` or `ESC [ 2   k`: RTL (right-to-left paragraph)

**Alacritty Storage**: Stored in `BidiFlags`:
- `NON_DEFAULT_PARA_DIR`: Set when non-default direction specified
- `RTL_PARA_DIR`: Set when RTL (2) specified

**Implementation Notes**:
```rust
match direction {
    0 => BidiMode::Explicit { default: true },
    1 => BidiMode::Explicit { ltr: true },
    2 => BidiMode::Explicit { rtl: true },
}
```

**Important**: Even when RTL paragraph direction is set, terminal still defaults to LTR base level (level 0) unless overridden. This is because terminals are left-aligned.

## CSI ? 2501 h / CSI ? 2501 l

**Auto-detection Mode**

- `CSI ? 2501 h` or `ESC [ ? 2501 h`: Enable auto-detection of paragraph direction
- `CSI ? 2501 l` or `ESC [ ? 2501 l`: Disable auto-detection

**Behavior**: When enabled, paragraph direction is determined from the first strong character in the text:
- First strong LTR char (L) → LTR paragraph (level 0)
- First strong RTL char (R, AL) → RTL paragraph (level 1)

**Alacritty Storage**: Stored in `BidiFlags::AUTO_PARA_DIR`

**Implementation**:
```rust
BidiMode::Auto { .. } => {
    use_auto_detect = true;
    break; // Auto mode takes precedence
}

// In UAX#9:
let para_level = if use_auto_detect {
    None // Let BidiInfo::new() detect automatically
} else {
    Some(Level::ltr()) // Default to LTR
};
```

**Priority**: Auto-detection takes precedence over explicit SCP settings.

## CSI ? 2500 h / CSI ? 2500 l

**Box Mirroring Mode**

- `CSI ? 2500 h` or `ESC [ ? 2500 h`: Enable box-drawing character mirroring for RTL
- `CSI ? 2500 l` or `ESC [ ? 2500 l`: Disable box mirroring

**Purpose**: When enabled, box-drawing characters should be horizontally mirrored in RTL context.

**Examples**:
- `┌` (top-left corner) becomes `┐` (top-right corner) in RTL
- `└` (bottom-left) becomes `┘` (bottom-right) in RTL
- `├` (left T) becomes `┤` (right T) in RTL

**Alacritty Status**: Flag stored in `BidiFlags::BOX_MIRRORING`, but rendering not yet implemented.

**UAX#9 Reference**: This corresponds to rule L4 (mirror brackets) in UAX#9, extended to box-drawing characters.

## CSI ? 1243 h / CSI ? 1243 l

**Arrow Key Swapping**

- `CSI ? 1243 h` or `ESC [ ? 1243 h`: Enable swapping left/right arrow keys in RTL mode
- `CSI ? 1243 l` or `ESC [ ? 1243 l`: Disable arrow key swapping

**Purpose**: When enabled and paragraph direction is RTL:
- Left arrow key moves right (logical direction)
- Right arrow key moves left (logical direction)

This provides logical navigation in RTL context.

**Alacritty Status**: Flag stored, behavior not yet implemented in input handling.

**Implementation Note**: This affects input processing, not display. The event handler should check this flag and the current paragraph direction before generating cursor movement.

## Sequence Interaction

### Priority Order

When multiple sequences affect the same line:

1. **Auto-detection** (CSI ? 2501 h) takes highest priority
2. **Explicit SCP** (CSI SP k) takes effect if auto-detection disabled
3. **Default LTR** used if neither auto nor explicit set

### Mode Combinations

Valid combinations:
- BiDi Implicit Mode (CSI 8 h) + Auto-detection (CSI ? 2501 h) ✅
- BiDi Implicit Mode (CSI 8 h) + Explicit SCP (CSI 1/2 SP k) ✅
- Box Mirroring (CSI ? 2500 h) + BiDi Implicit Mode ✅
- Arrow Swapping (CSI ? 1243 h) + RTL paragraph ✅

Invalid/redundant:
- Auto-detection + Explicit SCP (auto takes precedence, explicit ignored)

## Storage in Cell Flags

```rust
bitflags! {
    pub struct BidiFlags: u8 {
        const EXPLICIT_DIRECTION = 0b0000_0001;
        const NON_DEFAULT_PARA_DIR = 0b0000_0010;
        const RTL_PARA_DIR = 0b0000_0100;
        const AUTO_PARA_DIR = 0b0000_1000;
        const BOX_MIRRORING = 0b0001_0000;
    }
}
```

Arrow key swapping is stored separately (terminal-wide setting, not per-cell).

## Testing Sequences

### Test 1: Basic BiDi Enable
```
echo -e "\e[8h"  # Enable BiDi mode
echo "שלום"      # Should display reversed
echo -e "\e[8l"  # Disable BiDi mode
```

### Test 2: Explicit Paragraph Direction
```
echo -e "\e[8h\e[2 k"  # Enable BiDi + RTL paragraph
echo "hello שלום world"
echo -e "\e[1 k"       # Switch to LTR paragraph
```

### Test 3: Auto-detection
```
echo -e "\e[8h\e[?2501h"  # Enable BiDi + auto-detect
echo "שלום hello"         # Should detect RTL from first char
echo "hello שלום"         # Should detect LTR from first char
```

## Terminal-wg Specification Links

- Main spec: https://terminal-wg.pages.freedesktop.org/bidi/
- Escape sequences: https://terminal-wg.pages.freedesktop.org/bidi/recommendation/escape-sequences.html
- Conformance levels: https://terminal-wg.pages.freedesktop.org/bidi/recommendation/conformance.html

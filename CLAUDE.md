# RTL/BiDi Support Implementation

## Goal

Add Right-to-Left (RTL) and Bidirectional (BiDi) text support to Alacritty, enabling proper rendering of Hebrew, Arabic, and other RTL languages.

## Current Status

**Problem**: Alacritty displays RTL text incorrectly. For example, "שלום" (shalom) appears as "םולש" (reversed).

**Solution**: Port the BiDi implementation from closed PR #7872, which follows the terminal-wg BiDi specification.

## Implementation Approach

### What We're Porting

Implementing support for [terminal-wg's BiDi draft proposal](https://terminal-wg.pages.freedesktop.org/bidi/) - the same standard used by GNOME Terminal (VTE) since 2019.

**Key characteristics**:
- ✅ Feature-gated behind `bidi_draft` flag (optional, safe)
- ✅ No performance impact (verified by maintainer)
- ✅ Minimal changes (4 files, ~200 lines)
- ✅ No new dependencies (VTE 0.15.0 already includes SCP support)

### Changes Required

1. **alacritty_terminal/Cargo.toml**
   - Add `bidi_draft` feature flag
   - VTE dependency: Already at 0.15.0 (includes SCP from vte#112)

2. **alacritty_terminal/src/term/cell.rs**
   - Add `BidiFlags` (internal bitflags)
   - Add `BidiDir` enum (LTR, RTL, Default)
   - Add `BidiMode` enum (directionality modes)
   - Add `Cell::bidi_mode()` method
   - Add `Cell::bidi_box_mirroring()` method

3. **alacritty_terminal/src/term/mod.rs**
   - Add `Term::bidi_disable_arrow_key_swapping` field

## Development Guidelines

From Alacritty's CONTRIBUTING.md, distilled for this implementation:

### Architecture Principles
- **Separation of concerns**: `alacritty_terminal` (core terminal logic) vs. `alacritty` (UI/rendering)
  - Terminal layer: Data storage, protocol handling, logical order
  - UI layer: Visual transformations, rendering, display order
- **Avoid fragmentation**: Ensure `alacritty_terminal` consumers can reuse BiDi logic
- **Semver compliance**: Bump `alacritty_terminal` version according to semver when adding public APIs

### Code Quality
- **Style**: Follow Rust compiler style guidelines, use `rustfmt`
- **Comments**: All comments fully punctuated with trailing period
- **Documentation**: Rustdoc for all public APIs
- **MSRV**: Maintain Minimum Supported Rust Version, avoid unnecessary bumps

### Performance
- **Zero-cost abstractions**: Feature flags eliminate code when disabled
- **Benchmarking**: Test with vtebench for throughput, typometer for latency
- **Maintainer requirement**: "No performance impact" (verified by chrisduerr for original PR)

### Testing Requirements
- **Unit tests**: For all new functions (BiDi detection, reordering logic)
- **Integration tests**: Terminal escape sequences (modes 8, 1243, 2500, 2501, SCP)
- **Reference tests**: Visual output correctness
- **All tests must pass**: Both with and without `bidi_draft` feature enabled

### Submission Guidelines
- **Active communication**: Respond promptly to maintainer feedback
- **Clear PRs**: Document what changed, why, and how to test
- **Incremental approach**: Submit working features, not incomplete implementations
- **CHANGELOG.md**: Document user-facing changes following [keepachangelog.com](https://keepachangelog.com) format
  - Add entry under "Unreleased" section
  - Use user-perspective language (not technical implementation details)
  - Example: "Added support for bidirectional text (Hebrew, Arabic, etc.) via `bidi_draft` feature flag"
- **Pre-submission checklist**:
  - Run `cargo test` (all tests must pass)
  - Run `rustfmt` on all modified files
  - Test with both `--features bidi_draft` and without
  - Add unit tests for new functions
  - Performance benchmark if touching hot paths

## Quick Reference (from PR #7872)

**Core enums**:
- `BidiDir`: LTR | RTL | Default
- `BidiMode`: Explicit/Implicit/Auto (each with `BidiDir` field)
- `BidiFlags`: bitflags (EXPLICIT_DIRECTION, NON_DEFAULT_PARA_DIR, RTL_PARA_DIR, AUTO_PARA_DIR, BOX_MIRRORING)

**New Cell methods**:
- `bidi_mode() -> BidiMode` (public API)
- `bidi_box_mirroring() -> bool` (public API)
- Internal: `bidi_flags()`, `set_bidi_flags()`, `insert_bidi_flag()`, `remove_bidi_flag()`

**CellExtra field**: `bidi_flags: BidiFlags` (behind `#[cfg(feature = "bidi_draft")]`)

See PR #7872 for full implementation details.

## Relevant Links

### Original Work
- **Original PR**: [#7872](https://github.com/alacritty/alacritty/pull/7872) - Closed due to lack of communication, not technical issues
- **VTE PR**: [alacritty/vte#112](https://github.com/alacritty/vte/pull/112) - ✅ Merged (SCP control support)

### Related Issues
- **Main BiDi Issue**: [#663](https://github.com/alacritty/alacritty/issues/663) - Community requesting BiDi support (2017-)
- **RTL Text Issue**: [#2357](https://github.com/alacritty/alacritty/issues/2357) - RTL text scrambled

### Specifications
- **terminal-wg BiDi Spec**: https://terminal-wg.pages.freedesktop.org/bidi/
- **terminal-wg Implementations**: https://terminal-wg.pages.freedesktop.org/bidi/implementations.html

### Maintainer Feedback

From PR #7872:
- chrisduerr (maintainer): "no performance impact" ✅
- chrisduerr: "If someone else is interested in upstreaming this, please open a new PR" ✅
- chrisduerr: "I'm definitely happy to collaborate if that avoids fragmentation between alacritty_terminal consumers" ✅

## Why This Was Closed

**Not technical reasons** - Author (MoSal) didn't respond to maintainer's collaboration offer for 11 months.

## Success Criteria

1. Hebrew/Arabic text displays in correct order
2. Passes existing test suite
3. No performance regression
4. Feature documentation in place
5. Maintainer approval and merge

## Testing Plan

1. Enable `bidi_draft` feature in Cargo.toml
2. Build Alacritty with feature enabled
3. Test with Hebrew text: `echo "שלום"`
4. Test with Arabic text
5. Test mixed LTR/RTL text
6. Run existing test suite
7. Performance benchmarks

## Required Fixes to PR #7872

Before porting, we need to fix these issues found in the original PR:

### 1. Remove CancelBold Quirk (BLOCKER) ✅ FIXED
**Issue**: PR changes SGR 22 behavior when `bidi_draft` enabled:
- Original PR: `Attr::CancelBold` (SGR 22) → removes all underlines + sets double underline
- PR claims: "behavior defined in ECMA-48" and "for VTE compatibility"

**Why it's wrong (verified):**
- **ECMA-48 Standard**: SGR 21 = double underline, SGR 22 = cancel bold/dim (since 1984)
- **VTE Source Code**: SGR 22 only cancels bold/dim (no underline remapping)
- **VTE BiDi Code**: Does NOT touch SGR codes or text attributes
- **terminal-wg BiDi Spec**: No mention of SGR codes or underlines
- **Historical Context**: Linux kernel had SGR 21 bug (1992-2018), but that's unrelated to SGR 22
- **PR Discussion**: This change was NEVER reviewed or discussed by maintainer

**Impact:**
- Breaks standard terminal sequences (e.g., `\e[1m Bold \e[22m Normal` → adds double underline)
- Affects ALL text when feature enabled (not BiDi-specific)
- No terminal implements this behavior
- No specification requires this

**Root cause**: Author confused SGR 21/22 due to Linux kernel's historical bug with SGR 21

**Fix Applied**: ✅ The buggy CancelBold block was NOT ported. SGR 22 remains standard-compliant (line 1903).

### 2. Complete Missing Tests ⏳ PENDING
**Issue**: Tests exist for modes/directions but not for:
- Box mirroring escape sequences (CSI ? 2500 h/l)
- Arrow key swapping escape sequences (CSI ? 1243 h/l)
- SCP sequences for paragraph direction
- **Status**: Implementation complete, tests to be added in Phase 4

### 3. Add Documentation (Optional) ✅ COMPLETE
**Issue**: Public API lacks rustdoc:
- `BidiDir`, `BidiMode` enums
- `bidi_mode()`, `bidi_box_mirroring()` methods
- `bidi_disable_arrow_key_swapping` field
- **Fix Applied**: ✅ All public API now has rustdoc comments

## Progress

### Phase 1: Infrastructure ✅ COMPLETE

**Implemented:**
- ✅ Added `bidi_draft` feature flag to alacritty_terminal/Cargo.toml
- ✅ Added BiDi types to cell.rs:
  - `BidiFlags` bitflags (EXPLICIT_DIRECTION, NON_DEFAULT_PARA_DIR, RTL_PARA_DIR, AUTO_PARA_DIR, BOX_MIRRORING)
  - `BidiDir` enum (LTR, RTL, Default)
  - `BidiMode` enum (Explicit, Implicit, Auto)
- ✅ Added `bidi_flags` field to `CellExtra` with `#[serde(default)]`
- ✅ Added Cell methods:
  - Public: `bidi_mode()`, `bidi_box_mirroring()`
  - Internal: `bidi_flags()`, `set_bidi_flags()`, `insert_bidi_flag()`, `remove_bidi_flag()`
- ✅ Added `bidi_disable_arrow_key_swapping` field to Term struct
- ✅ Added rustdoc documentation to all public API
- ✅ All 264 tests pass with and without `bidi_draft` feature
- ✅ CancelBold quirk from PR #7872 was NOT ported (verified)

**Files Modified:**
- alacritty_terminal/Cargo.toml (line 16)
- alacritty_terminal/src/term/cell.rs (lines 39-328)
- alacritty_terminal/src/term/mod.rs (lines 331-333, 447-448)

### Phase 2: Escape Sequence Handlers ✅ COMPLETE

**Implemented:**
- ✅ Mode 8 (BDSM - BiDi Support Mode) handlers:
  - `set_mode()`: Enables implicit BiDi mode
  - `unset_mode()`: Enables explicit BiDi mode
- ✅ Private mode handlers (modes 1243, 2500, 2501):
  - Mode 1243: Arrow key swapping control
  - Mode 2500: Box mirroring toggle
  - Mode 2501: Auto paragraph direction detection
- ✅ SCP (Select Character Path) handler:
  - Default/LTR/RTL paragraph direction control
- ✅ All handlers feature-gated behind `bidi_draft`
- ✅ All 264 tests pass
- ✅ SGR 22 (CancelBold) remains unmodified

**Files Modified:**
- alacritty_terminal/src/term/mod.rs (lines 1940-2166)

### Phase 3: Rendering Integration ✅ COMPLETE

**Architecture Decision:**
Following Alacritty's existing pattern (terminal=data, UI=transformations), BiDi reordering is implemented in the UI layer, with reusable logic in `alacritty_terminal/src/term/bidi.rs`.

**Implementation Strategy:**
BiDi reordering is applied after cell collection and before terminal drop, using terminal grid data to determine reordering while updating RenderableCell character data in place.

**Performance characteristics:**
- Feature disabled: Zero runtime cost (compile-time elimination)
- Pure LTR text: Minimal overhead (fast line_needs_bidi checks)
- With RTL text: Applies Unicode BiDi algorithm only to affected lines

**Implemented:**

1. **alacritty_terminal/Cargo.toml**
   - ✅ Add `unicode-bidi = { version = "0.3", optional = true }` dependency
   - ✅ Update feature: `bidi_draft = ["dep:unicode-bidi"]`

2. **alacritty_terminal/src/term/bidi.rs** (NEW FILE)
   - ✅ `line_needs_bidi(row: &Row<Cell>) -> bool` - Fast RTL/BiDi detection
   - ✅ `reorder_line(row: &Row<Cell>) -> Vec<Cell>` - Apply Unicode BiDi Algorithm with byte-to-char index mapping
   - ✅ Unit tests for pure LTR, pure RTL text (all tests pass)

3. **alacritty_terminal/src/term/mod.rs**
   - ✅ Add `pub mod bidi;` (feature-gated)

4. **alacritty/Cargo.toml**
   - ✅ Add `bidi_draft = ["alacritty_terminal/bidi_draft"]` feature

5. **alacritty/src/display/mod.rs**
   - ✅ Added BiDi reordering logic after cell collection (lines 814-847)
   - ✅ Groups cells by line, checks if BiDi needed, reorders character data
   - ✅ Maintains viewport coordinates and rendering order
   - ✅ Feature-gated behind `#[cfg(feature = "bidi_draft")]`

**Files Modified:**
- alacritty_terminal/Cargo.toml (lines 16, 30)
- alacritty_terminal/src/term/mod.rs (line 33)
- alacritty_terminal/src/term/bidi.rs (new file, ~180 lines, with tests)
- alacritty/Cargo.toml (line 103)
- alacritty/src/display/mod.rs (lines 814-847)

**Test Results:**
- ✅ All 137 tests pass with `bidi_draft` feature
- ✅ All tests pass without feature
- ✅ Zero-cost abstraction verified (no code when disabled)

### Phase 4: Testing & Documentation ⏳ PENDING

**Required:**
- Add tests for BiDi escape sequences
- Test with real RTL text (Hebrew: `echo "שלום"`)
- Test mixed LTR/RTL text
- Performance regression testing

### Phase 5: PR Submission ⏳ PENDING

**Pre-submission Checklist:**
- [ ] All tests pass: `cargo test`
- [ ] Tests pass with feature: `cargo test --features bidi_draft`
- [ ] Code formatted: `rustfmt` applied to all modified files
- [ ] Unit tests added for new functions
- [ ] Performance benchmarked (no regression)
- [ ] CHANGELOG.md updated with user-facing description
- [ ] Public APIs documented with rustdoc
- [ ] Test with real RTL text (Hebrew, Arabic)

**PR Submission:**
- Submit PR to alacritty/alacritty on GitHub
- Include in PR description:
  - What: BiDi/RTL text support via `bidi_draft` feature flag
  - Why: Fixes issues #663 and #2357 (RTL text scrambled)
  - How to test: Enable feature, test with `echo "שלום"`, verify visual order
  - Performance: No impact when disabled, minimal overhead when enabled
  - Reference: Based on closed PR #7872 (updated with fixes)

**During Review:**
- Respond to maintainer feedback within 48 hours
- Address all requested changes promptly
- Avoid fragmentation with `alacritty_terminal` consumers
- Be prepared to iterate on design if needed

## Timeline

- Phase 1: Port changes from PR #7872 with fixes ✅ COMPLETE
- Phase 2: Escape sequence handlers ✅ COMPLETE
- Phase 3: Rendering integration ⏳ PENDING
- Phase 4: Testing & documentation ⏳ PENDING
- Phase 5: PR submission ⏳ PENDING

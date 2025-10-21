# External References

Comprehensive list of external resources for BiDi implementation.

## Official Specifications

### Terminal-wg BiDi Specification

**Main Page**
- https://terminal-wg.pages.freedesktop.org/bidi/
- Overview of BiDi support in terminal emulators
- Conformance levels and recommendations

**Recommendation**
- https://terminal-wg.pages.freedesktop.org/bidi/recommendation/
- Detailed technical requirements
- Best practices for implementation

**Escape Sequences**
- https://terminal-wg.pages.freedesktop.org/bidi/recommendation/escape-sequences.html
- Complete list of BiDi control sequences
- CSI 8, SCP, 2500, 2501, 1243 specifications

**Conformance Levels**
- https://terminal-wg.pages.freedesktop.org/bidi/recommendation/conformance.html
- Level 1 (Implicit Mode) - what Alacritty implements
- Level 2 (Explicit Mode) - future consideration
- Level 3 (Full Mode) - advanced features

### Unicode BiDi Algorithm (UAX#9)

**Current Version**
- https://www.unicode.org/reports/tr9/
- Unicode Standard Annex #9
- Defines the Bidirectional Algorithm

**Versioned Specs**
- https://www.unicode.org/reports/tr9/tr9-46.html (UAX#9 Version 15.1.0)
- Always check version compatibility with unicode-bidi crate

**Character Database**
- https://www.unicode.org/Public/UCD/latest/
- Character type classifications
- BiDi class property for all Unicode characters

**FAQ**
- https://www.unicode.org/faq/bidi.html
- Common questions about BiDi
- Clarifications on algorithm behavior

## Implementation References

### VTE (GNOME Terminal Backend)

**BiDi Implementation (C++)**
- https://gitlab.gnome.org/GNOME/vte/-/blob/master/src/bidi.cc
- https://gitlab.gnome.org/GNOME/vte/-/blob/master/src/bidi.hh
- Reference implementation in C++
- Similar architecture to Alacritty

**VTE PR #112 (SCP Support)**
- https://github.com/alacritty/vte/pull/112
- Merged SCP escape sequence support
- Relevant for Alacritty's vte fork

### mlterm (Multi-lingual Terminal)

**BiDi Support**
- https://sourceforge.net/projects/mlterm/
- One of the first terminals with BiDi support
- Good reference for edge cases

### unicode-bidi Rust Crate

**Documentation**
- https://docs.rs/unicode-bidi/
- API reference for the crate Alacritty uses
- Examples and usage patterns

**Source Code**
- https://github.com/servo/unicode-bidi
- Originally from Servo browser engine
- Well-tested implementation

**Crates.io**
- https://crates.io/crates/unicode-bidi
- Version history and dependencies

## Alacritty Context

### Related Pull Requests

**PR #7872 (Closed - Base for Current Work)**
- https://github.com/alacritty/alacritty/pull/7872
- Initial BiDi implementation attempt
- Provides context and discussion

**PR #8137 (Current/Future)**
- Check for active BiDi PR
- May supersede #7872

### Related Issues

**Issue #663 (BiDi Support Request)**
- https://github.com/alacritty/alacritty/issues/663
- Original feature request from 2017
- Long discussion with use cases

**Issue #2357 (RTL Text Scrambled)**
- https://github.com/alacritty/alacritty/issues/2357
- Bug report about RTL rendering
- User-reported issues

**Issue #3146 (Arabic Text)**
- https://github.com/alacritty/alacritty/issues/3146
- Specific to Arabic text rendering
- Shaping vs BiDi discussion

### VTE Fork

**Alacritty's VTE Fork**
- https://github.com/alacritty/vte
- Escape sequence parser used by Alacritty
- SCP support merged in PR #112

## Testing Resources

### Test Files and Data

**Hebrew Text Samples**
- https://www.w3.org/International/tests/repo/results/bidi-text-rendering
- W3C BiDi text rendering tests
- Browser-focused but applicable

**Arabic Text Samples**
- https://www.unicode.org/udhr/d/udhr_arb.html
- Universal Declaration of Human Rights in Arabic
- Good for real-world testing

**BiDi Test Suite**
- https://www.unicode.org/Public/UCD/latest/ucd/BidiTest.txt
- Official Unicode BiDi conformance tests
- Character-level test cases

### Performance Benchmarks

**vtebench**
- https://github.com/alacritty/vtebench
- Terminal emulator benchmark
- Useful for performance regression testing

**typometer**
- https://github.com/pavelfatin/typometer
- Input latency measurement
- Verify BiDi doesn't add lag

## Educational Resources

### Tutorials and Guides

**W3C BiDi Tutorial**
- https://www.w3.org/International/tutorials/bidi-xhtml/
- Web-focused but concepts apply
- Good introduction to BiDi concepts

**Joel on Software - BiDi**
- https://www.joelonsoftware.com/2003/10/08/the-absolute-minimum-every-software-developer-absolutely-positively-must-know-about-unicode-and-character-sets-no-excuses/
- General Unicode introduction
- BiDi as part of Unicode

**ICU User Guide - BiDi**
- https://unicode-org.github.io/icu/userguide/transforms/bidi/
- IBM's ICU library BiDi documentation
- Detailed algorithm explanations

### Papers and Articles

**"Bidirectional Text Rendering in Terminal Emulators"**
- Search academic databases
- May find implementation experiences

**Terminal-wg Mailing List Archives**
- Discussion of BiDi specification decisions
- Rationale for design choices

## Tools and Utilities

### BiDi Debuggers

**Unicode BiDi Visualizer**
- https://www.unicode.org/cldr/utility/bidi.jsp
- Online tool to visualize BiDi algorithm
- Shows levels and runs

**FriBidi (Free Implementation)**
- https://github.com/fribidi/fribidi
- C library implementation
- Includes testing tools

### Text Editors with BiDi

**Vim**
- `:set rightleft` and `:set arabic`
- Test BiDi with real editor

**Emacs**
- BiDi support built-in
- Good for integration testing

**VS Code**
- Modern editor with BiDi
- Reference for expected behavior

## Language-Specific Resources

### Hebrew

**Hebrew Alphabet**
- https://en.wikipedia.org/wiki/Hebrew_alphabet
- Character reference
- Unicode blocks: U+0590–U+05FF

**Hebrew Computing**
- https://en.wikipedia.org/wiki/Hebrew_and_computers
- History and technical details

### Arabic

**Arabic Alphabet**
- https://en.wikipedia.org/wiki/Arabic_alphabet
- Character reference
- Unicode blocks: U+0600–U+06FF, U+0750–U+077F

**Arabic Text Shaping**
- https://www.unicode.org/reports/tr53/
- Separate from BiDi but often confused
- Important distinction

### Other RTL Scripts

**Syriac**
- Unicode block: U+0700–U+074F

**Thaana (Maldivian)**
- Unicode block: U+0780–U+07BF

**N'Ko (West African)**
- Unicode block: U+07C0–U+07FF

## Community and Support

### Forums and Discussion

**Alacritty Discussions**
- https://github.com/alacritty/alacritty/discussions
- Ask questions, share progress

**Reddit - r/alacritty**
- https://reddit.com/r/alacritty
- Community discussions

**Stack Overflow**
- Tag: [unicode-bidi]
- Tag: [right-to-left]

### Experts and Maintainers

**Behdad Esfahbod**
- BiDi and text rendering expert
- Contributor to many open-source projects
- @behdad on GitHub

**Khaled Hosny**
- Arabic text expert
- HarfBuzz maintainer

## Standards Organizations

**Unicode Consortium**
- https://www.unicode.org/
- Maintains UAX#9 and character database

**W3C Internationalization**
- https://www.w3.org/International/
- Web standards for BiDi and i18n

**freedesktop.org**
- https://www.freedesktop.org/
- Hosts terminal-wg specification

## Related Technologies

### Text Shaping (Separate from BiDi)

**HarfBuzz**
- https://harfbuzz.github.io/
- Text shaping engine
- Handles Arabic joining, not BiDi

**Unicode Text Segmentation (UAX#29)**
- https://www.unicode.org/reports/tr29/
- Grapheme clusters
- Related to BiDi boundaries

### Font Technologies

**OpenType Features**
- https://docs.microsoft.com/en-us/typography/opentype/spec/
- RTL-specific features
- `rtla`, `rtlm` features

## Rust-Specific Resources

**Rust i18n Working Group**
- https://github.com/rust-lang/rfcs/pull/2457
- Internationalization efforts in Rust

**servo/unicode-bidi**
- Already listed above
- Primary resource for Rust BiDi

**rusttype / ab_glyph**
- Font rendering crates
- May need integration with BiDi

## Version History

**UAX#9 Versions**
- Version 6.3.0: Added isolating run sequences
- Version 7.0.0: Bracket matching (N0 rule)
- Current: Check unicode.org for latest

**terminal-wg Spec Versions**
- Check git history at freedesktop.org
- Spec is relatively stable

## Quick Reference Links

Most commonly needed during development:

1. **Terminal-wg Escape Sequences**: https://terminal-wg.pages.freedesktop.org/bidi/recommendation/escape-sequences.html
2. **UAX#9 Main Spec**: https://www.unicode.org/reports/tr9/
3. **unicode-bidi Docs**: https://docs.rs/unicode-bidi/
4. **VTE BiDi Code**: https://gitlab.gnome.org/GNOME/vte/-/blob/master/src/bidi.cc
5. **Alacritty Issue #663**: https://github.com/alacritty/alacritty/issues/663

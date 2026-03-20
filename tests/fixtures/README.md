# tests/fixtures

This directory collects fixtures for integration tests and manual verification.

## Overview

### `dir/left/`, `dir/right/`
- Purpose:
  - Verify the basic behavior of directory mode
  - Verify listing for `Added` / `Deleted` / `Modified` / `Unchanged`
- Main usage:
  - `tests/directory_diff.rs`

### `multi/left/`, `multi/right/`
- Purpose:
  - Manual verification of directory diffs with multiple files
  - Check how diffs look across README / config / nested docs / `src`
- Main usage:
  - Mainly for manual verification
- Expected diff summary:
  - `README.md`: unchanged
  - `config.toml`: modified
  - `src/app.rs`: modified (multiple hunks)
  - `src/notes.txt`: modified (multiple hunks)
  - `docs/only_left.md`: deleted
  - `docs/only_right.md`: added

### `encoding/`
- Purpose:
  - Verify UTF-8 BOM and non-UTF-8 display behavior
  - Check pane titles, status, and load policy with mixed encodings
- Contents:
  - `left/utf8_bom.txt`
    - UTF-8 BOM + `hello\n`
  - `right/utf8_bom.txt`
    - UTF-8 without BOM + `hello\n`
  - `left/non_utf8.bin`
    - Non-UTF-8 bytes without NUL
  - `right/non_utf8.bin`
    - UTF-8 text `hello\n`
- Main usage:
  - Manual verification
- Example:
  - `cargo run --release -- tests/fixtures/encoding/left tests/fixtures/encoding/right`

### `whitespace/`
- Purpose:
  - Manual verification of whitespace-only diffs
  - Compare behavior before and after `ignore whitespace`
- Contents:
  - `left/indent.txt` / `right/indent.txt`
    - Only indentation width differs
  - `left/trailing.txt` / `right/trailing.txt`
    - Only trailing spaces differ
  - `left/tab.txt` / `right/tab.txt`
    - Only tabs vs spaces differ
  - `left/token.txt` / `right/token.txt`
    - Includes both whitespace changes and non-whitespace changes such as `int` vs `double`
- Main usage:
  - Manual verification
- Example:
  - `PONTIS_WHITESPACE_POLICY=compare cargo run -- tests/fixtures/whitespace/left tests/fixtures/whitespace/right`
  - `PONTIS_WHITESPACE_POLICY=ignore cargo run -- tests/fixtures/whitespace/left tests/fixtures/whitespace/right`
  - `PONTIS_WHITESPACE_POLICY=ignore PONTIS_INLINE_DIFF=false cargo run -- tests/fixtures/whitespace/left tests/fixtures/whitespace/right`

### `screenshot/left/`, `screenshot/right/`
- Purpose:
  - Reproduce screenshots for the README and docs
  - Keep a fixture that can show `= / M / R / A / D` together in the file list
- Main usage:
  - Mainly for manual verification
  - Sample image generation for the README
- Expected diff summary:
  - `README.md`: unchanged
  - `Cargo.toml`: modified
  - `src/app.rs`: modified (multiple hunks)
  - `src/render/status.rs`: modified (multiple hunks)
  - `src/rename_target.rs -> src/renamed_target.rs`: renamed / moved (README `R` sample)
  - `src/git_bridge.rs -> src/git/difftool.rs`: renamed / moved
  - `docs/architecture.md`: deleted
  - `docs/release.md`: added
  - `src/theme.rs`: added
- Example:
  - `cargo run -- tests/fixtures/screenshot/left tests/fixtures/screenshot/right`
  - For the README screenshot, selecting `src/renamed_target.rs` makes it easier to show `R`
  - If you also want to show `*`, apply one hunk merge in `src/app.rs`

## Update Policy
- When adding a new fixture, also add its purpose and main usage here
- Even for manual-only fixtures, leave at least a minimal note about what they are meant to verify

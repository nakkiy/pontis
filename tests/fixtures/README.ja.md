# tests/fixtures

このディレクトリは、統合テストと手動確認用の fixture をまとめたものです。

## 一覧

### `dir/left/`, `dir/right/`
- 目的:
  - directory mode の基本動作確認
  - `Added` / `Deleted` / `Modified` / `Unchanged` の一覧化確認
- 主な利用箇所:
  - `tests/directory_diff.rs`

### `multi/left/`, `multi/right/`
- 目的:
  - 複数ファイルを含む directory diff の手動確認
  - README / config / nested docs / src 配下の差分見え方確認
- 主な利用箇所:
  - 主に手動確認用
- 期待する差分概要:
  - `README.md`: unchanged
  - `config.toml`: modified
  - `src/app.rs`: modified（複数 hunk）
  - `src/notes.txt`: modified（複数 hunk）
  - `docs/only_left.md`: deleted
  - `docs/only_right.md`: added

### `encoding/`
- 目的:
  - UTF-8 BOM と non-UTF-8 の表示確認
  - mixed encoding 時の pane title / status / 読込ポリシー確認
- 内容:
  - `left/utf8_bom.txt`
    - UTF-8 BOM + `hello\n`
  - `right/utf8_bom.txt`
    - UTF-8（BOM なし）+ `hello\n`
  - `left/non_utf8.bin`
    - 非 UTF-8 バイト列（NUL なし）
  - `right/non_utf8.bin`
    - UTF-8 テキスト `hello\n`
- 主な利用箇所:
  - 手動確認用
- 実行例:
  - `cargo run --release -- tests/fixtures/encoding/left tests/fixtures/encoding/right`

### `whitespace/`
- 目的:
  - whitespace-only diff の手動確認
  - 将来の `ignore whitespace` 実装前後の比較確認
- 内容:
  - `left/indent.txt` / `right/indent.txt`
    - インデント幅だけが違う
  - `left/trailing.txt` / `right/trailing.txt`
    - 行末空白だけが違う
  - `left/tab.txt` / `right/tab.txt`
    - タブとスペースだけが違う
  - `left/token.txt` / `right/token.txt`
    - 空白差分に加えて `int` と `double` の非空白差分を含む
- 主な利用箇所:
  - 手動確認用
- 実行例:
  - `PONTIS_COMPARE_WHITESPACE=compare cargo run -- tests/fixtures/whitespace/left tests/fixtures/whitespace/right`
  - `PONTIS_COMPARE_WHITESPACE=ignore cargo run -- tests/fixtures/whitespace/left tests/fixtures/whitespace/right`
  - `PONTIS_COMPARE_WHITESPACE=ignore PONTIS_COMPARE_INLINE_DIFF=false cargo run -- tests/fixtures/whitespace/left tests/fixtures/whitespace/right`

### `screenshot/left/`, `screenshot/right/`
- 目的:
  - README / docs 向けスクリーンショットの再現
  - file list に `= / M / R / A / D` を同時表示しやすい fixture を固定
- 主な利用箇所:
  - 主に手動確認用
  - README の sample image 作成用
- 期待する差分概要:
  - `README.md`: unchanged
  - `Cargo.toml`: modified
  - `src/app.rs`: modified（複数 hunk）
  - `src/render/status.rs`: modified（複数 hunk）
  - `src/rename_target.rs -> src/renamed_target.rs`: renamed / moved（README 用の `R` サンプル）
  - `src/git_bridge.rs -> src/git/difftool.rs`: renamed / moved
  - `docs/architecture.md`: deleted
  - `docs/release.md`: added
  - `src/theme.rs`: added
- 実行例:
  - `cargo run -- tests/fixtures/screenshot/left tests/fixtures/screenshot/right`
  - README 用スクリーンショットでは `src/renamed_target.rs` を選ぶと `R` を出しやすい
  - `*` も見せたい場合は `src/app.rs` で hunk merge を 1 回適用する

## 更新方針
- 新しい fixture を追加したら、この一覧にも目的と主な利用箇所を追記する
- 手動確認専用 fixture でも、何を見るためのものかを最低限残す

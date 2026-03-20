# pontis

`pontis` は Rust + `ratatui` で構築された **TUI diff / merge ツール**です。

端末から離れずに、**ディレクトリ比較・Git差分確認・hunk単位マージ**を完結させることを目的としています。

---

## 特徴
![screenshot](screenshot.png)

* ディレクトリを再帰的に比較
* hunk 単位での双方向マージ
* undo / redo 対応
* side-by-side diff（inline diff対応）
* `$EDITOR` 連携による部分編集
* Git（working tree / index / revision）対応

---

## インストール

### 事前ビルド済みバイナリ

GitHub Releases から利用中のプラットフォーム向けアーカイブを取得し、展開後の
`pontis` バイナリを `PATH` の通った場所へ配置してください。

### ソースからインストール

```bash
cargo install --git https://github.com/nakkiy/pontis pontis
```

---

## 使い方

### ローカル比較

```bash
pontis <left> <right>
```

### 設定ファイルを指定して起動

```bash
pontis --config /path/to/config.toml <left> <right>
```

---

## Git モード

### working tree vs HEAD

```bash
pontis git
pontis git --repo /path/to/repo
```

### index vs HEAD

```bash
pontis git --staged
```

### working tree vs revision

```bash
pontis git --rev <rev>
```

### index vs revision

```bash
pontis git --rev <rev> --staged
```

### revision pair

```bash
pontis git --diff <rev1> <rev2>
```

詳細: [Git 連携ガイド](GIT_INTEGRATION.ja.md)

---

## git difftool 連携

設定:
```bash
git config --global diff.tool pontis
git config --global difftool.prompt false
git config --global difftool.pontis.cmd \
  'pontis git --repo "$PWD" \
    --diff "$PONTIS_GIT_DIFFTOOL_LEFT_REV" "$PONTIS_GIT_DIFFTOOL_RIGHT_REV" \
    --difftool-left-dir "$LOCAL" \
    --difftool-right-dir "$REMOTE"'
```

実行:

```bash
REV1=HEAD~1
REV2=HEAD

PONTIS_GIT_DIFFTOOL_LEFT_REV="$REV1" \
PONTIS_GIT_DIFFTOOL_RIGHT_REV="$REV2" \
git difftool --tool pontis --dir-diff "$REV1" "$REV2"
```

頻繁に使うなら関数にしておくと楽です

```bash
git-pontis-diff() {
  local rev1="$1"
  local rev2="$2"
  PONTIS_GIT_DIFFTOOL_LEFT_REV="$rev1" \
  PONTIS_GIT_DIFFTOOL_RIGHT_REV="$rev2" \
  git difftool --tool pontis --dir-diff "$rev1" "$rev2"
}
```

---

## lazygit 連携

```yaml
customCommands:
  - key: "D"
    context: "commits"
    description: "選択コミットと親を pontis で比較"
    command: >
      PONTIS_GIT_DIFFTOOL_LEFT_REV={{.SelectedLocalCommit.Hash}}^
      PONTIS_GIT_DIFFTOOL_RIGHT_REV={{.SelectedLocalCommit.Hash}}
      git difftool --tool pontis --dir-diff
      {{.SelectedLocalCommit.Hash}}^ {{.SelectedLocalCommit.Hash}}
    output: terminal

  - key: "R"
    context: "commits"
    description: "選択レンジを pontis で比較"
    command: >
      PONTIS_GIT_DIFFTOOL_LEFT_REV={{.SelectedCommitRange.From}}
      PONTIS_GIT_DIFFTOOL_RIGHT_REV={{.SelectedCommitRange.To}}
      git difftool --tool pontis --dir-diff
      {{.SelectedCommitRange.From}} {{.SelectedCommitRange.To}}
    output: terminal
```

---

## UI 概要

### 左ペイン（file list）

* ファイル一覧
* status フィルタ

### 右ペイン（diff）

* 横並び差分表示
* hunk単位操作
* スクロール・ナビゲーション

---

## status marker

| 記号  | 意味            |
| --- | ----------------- |
| `=` | unchanged         |
| `?` | pending（未評価） |
| `M` | modified          |
| `R` | renamed / moved   |
| `A` | added             |
| `D` | deleted           |
| `B` | binary            |
| `*` | dirty（未保存）   |

---

## キーバインド

### 共通

| key         | 動作             |
| ----------- | ---------------- |
| `q`         | 終了             |
| `alt+↓ / ↑` | 次 / 前の差分    |
| `alt+→ / ←` | hunk 適用        |
| `u / r`     | undo / redo      |
| `e / E`     | editor で開く    |
| `s / S`     | 保存             |

---

### file list

| key               | 動作             |
| ----------------- | ---------------- |
| `enter`           | diffへ           |
| `↑/↓`             | 移動             |
| `PageUp/PageDown` | 10件移動         |
| `←/→`             | 横スクロール     |
| `Home/End`        | 横端へ移動       |
| `A/M/D/R/=`       | フィルタ         |
| `f`               | フィルタリセット |

---

### diff

| key               | 動作               |
| ----------------- | ------------------ |
| `esc`             | file listへ戻る    |
| `↑/↓`             | スクロール         |
| `PageUp/PageDown` | 10行スクロール     |
| `←/→`             | 横スクロール       |
| `Home/End`        | 横端へ移動         |

---

## `$EDITOR` / 保存の制約

* `pontis git` / `pontis git --rev` は右側のみ書き込み可
* `pontis git --staged` は左右とも read-only
* `pontis git --diff` と `git difftool` 連携は左右とも read-only
* 外部 editor の変更は一度メモリへ取り込み、保存は `s` / `S` で明示的に行います
* 外部 editor から戻ると merge の undo / redo 履歴はクリアされます

---

## 設定

### config file

* `${XDG_CONFIG_HOME}/pontis/config.toml`
* `~/.config/pontis/config.toml`

```toml
backup_on_save = false
highlight_max_bytes = 524288
highlight_max_lines = 8000
theme = ""
inline_diff = true
line_ending_policy = "compare"
whitespace_policy = "compare"
line_numbers = false
line_ending_visibility = "hidden"
```

---

### 優先順位

* `CLI > 環境変数 > config file > default`

---

### 環境変数

* `PONTIS_BACKUP_ON_SAVE`
* `PONTIS_HIGHLIGHT_MAX_BYTES`
* `PONTIS_HIGHLIGHT_MAX_LINES`
* `PONTIS_THEME`
* `PONTIS_INLINE_DIFF`
* `PONTIS_LINE_ENDING_POLICY`
* `PONTIS_WHITESPACE_POLICY`
* `PONTIS_LINE_NUMBERS`
* `PONTIS_LINE_ENDING_VISIBILITY`

---

### custom assets

追加テーマや syntax 定義は次のディレクトリに配置できます。存在しない場合は標準設定だけで動作します。

- `themes/` には Sublime Text / `syntect` 互換テーマを置けます
- `themes/` に置いたテーマは `theme = "..."` で指定できます
- `syntaxes/` には Sublime Text / `syntect` 互換 syntax 定義を置けます
- `syntaxes/` に置いた定義は起動時に読み込まれます

```
~/.config/pontis/themes/
~/.config/pontis/syntaxes/
```

詳細: [設定リファレンス](CONFIG.ja.md)

---

## 制限

* `file/dir` 混在入力は非対応
* binary は diff 表示不可
* binary への hunk merge は不可
* UTF-8 のみ対応
* UTF-8 BOM は保存時に保持
* 大きいファイルは syntax highlight 無効

---

## 今後

* リロード機能
* file list のキーワードフィルタ
* キーバインドの設定カスタマイズ
* install script と配布自動化

---

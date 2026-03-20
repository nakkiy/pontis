# pontis 設定リファレンス

`pontis` の設定は、CLI / 環境変数 / 設定ファイルから与えられます。

優先順位:

* `CLI > 環境変数 > config file > default`

---

## 設定ファイルの場所

`pontis` は次の順で設定ファイルを探します。

* `${XDG_CONFIG_HOME}/pontis/config.toml`
* `~/.config/pontis/config.toml`

明示的に指定する場合:

```bash
pontis --config /path/to/config.toml <left> <right>
```

---

## TOML キー

現行の設定ファイルはフラットなキー形式です。

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

### `backup_on_save`

* 型: `bool`
* 既定値: `false`
* 保存時に既存ファイルのバックアップを作るかを切り替えます

### `highlight_max_bytes`

* 型: `usize`
* 既定値: `524288`
* これを超えるファイルでは syntax highlight を無効化します

### `highlight_max_lines`

* 型: `usize`
* 既定値: `8000`
* 行数がこれを超える場合も syntax highlight を無効化します

### `theme`

* 型: `string`
* 既定値: `""`
* 使用するテーマ名です
* 空文字の場合は既定テーマを選びます

### `inline_diff`

* 型: `bool`
* 既定値: `true`
* 行内差分の強調を有効化します

### `line_ending_policy`

* 型: `string`
* 既定値: `"compare"`
* 値:
  * `"compare"`: 改行コード差分を比較に含める
  * `"ignore"`: CR / LF / CRLF の差分を比較から除外する

### `whitespace_policy`

* 型: `string`
* 既定値: `"compare"`
* 値:
  * `"compare"`: 空白差分を比較に含める
  * `"ignore"`: 空白差分を比較から除外する

補足:

* `whitespace_policy = "ignore"` でも、inline diff は非空白の差分を強調します
* 空白だけの差分は inline diff では強調しません

### `line_numbers`

* 型: `bool`
* 既定値: `false`
* diff pane に行番号を表示します

### `line_ending_visibility`

* 型: `string`
* 既定値: `"hidden"`
* 値:
  * `"hidden"`: 表示しない
  * `"all"`: すべての行末記号を表示する
  * `"diff_only"`: 差分がある行だけ表示する

表示記号:

* `←`: `CR`
* `↓`: `LF`
* `↩`: `CRLF`

---

## 環境変数

次の環境変数で設定を上書きできます。

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

## custom assets

追加テーマや syntax 定義は次のディレクトリに置けます。

* `~/.config/pontis/themes/`
* `~/.config/pontis/syntaxes/`

補足:

* `themes/` には Sublime Text / `syntect` 互換テーマを配置できます
* `syntaxes/` には Sublime Text / `syntect` 互換 syntax 定義を配置できます
* ディレクトリが存在しない場合は標準アセットだけで動作します
* `themes/` に置いたテーマは `theme = "..."` で指定できます

---

## 例

### 空白差分を無視したい

```toml
whitespace_policy = "ignore"
inline_diff = true
```

この場合、比較では空白差分を無視しつつ、inline diff は非空白の差分だけを強調します。

### 行末記号を見たい

```toml
line_ending_visibility = "all"
```

### バックアップを残したい

```toml
backup_on_save = true
```

---

## 予定

長期的には設定ファイルをフラットキーからセクション形式へ移行する計画があります。

想定例:

```toml
[compare]
whitespace = "compare"
line_endings = "compare"
inline_diff = true

[view]
line_numbers = false
line_ending_visibility = "hidden"

[highlight]
theme = ""
max_bytes = 524288
max_lines = 8000

[save]
backup_on_save = false
```

現時点ではまだこの形式ではなく、上記は将来案です。

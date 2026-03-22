# pontis Configuration Reference

`pontis` accepts configuration from the CLI, environment variables, and a config file.

Priority:

* `CLI > environment variables > config file > default`

---

## Config File Location

`pontis` looks for a config file in the following locations:

* `${XDG_CONFIG_HOME}/pontis/config.toml`
* `~/.config/pontis/config.toml`

To specify one explicitly:

```bash
pontis --config /path/to/config.toml <left> <right>
```

---

## TOML Keys

The config file uses section-based TOML. 

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
create_backup = false
```

### `[compare].whitespace`

* Type: `string`
* Default: `"compare"`
* Values:
  * `"compare"`: include whitespace differences
  * `"ignore"`: ignore whitespace differences

Note:

* With `[compare].whitespace = "ignore"`, inline diff still highlights non-whitespace changes
* Whitespace-only changes are not highlighted in inline diff

### `[compare].line_endings`

* Type: `string`
* Default: `"compare"`
* Values:
  * `"compare"`: include line ending differences in comparison
  * `"ignore"`: ignore CR / LF / CRLF differences

### `[compare].inline_diff`

* Type: `bool`
* Default: `true`
* Enables inline diff highlighting

### `[view].line_numbers`

* Type: `bool`
* Default: `false`
* Shows line numbers in the diff pane

### `[view].line_ending_visibility`

* Type: `string`
* Default: `"hidden"`
* Values:
  * `"hidden"`: do not show line ending markers
  * `"all"`: show markers for every line
  * `"diff_only"`: show markers only where line endings differ

Displayed markers:

* `←`: `CR`
* `↓`: `LF`
* `↩`: `CRLF`

### `[highlight].theme`

* Type: `string`
* Default: `""`
* Selects the theme by name
* An empty string means the default theme selection is used

### `[highlight].max_bytes`

* Type: `usize`
* Default: `524288`
* Disables syntax highlighting for files larger than this

### `[highlight].max_lines`

* Type: `usize`
* Default: `8000`
* Also disables syntax highlighting when the file has more than this many lines

### `[save].create_backup`

* Type: `bool`
* Default: `false`
* Controls whether a backup is created before writing an existing file

---

## Environment Variables

The following environment variables can override config values:

* `PONTIS_SAVE_CREATE_BACKUP`
* `PONTIS_HIGHLIGHT_MAX_BYTES`
* `PONTIS_HIGHLIGHT_MAX_LINES`
* `PONTIS_HIGHLIGHT_THEME`
* `PONTIS_COMPARE_INLINE_DIFF`
* `PONTIS_COMPARE_LINE_ENDINGS`
* `PONTIS_COMPARE_WHITESPACE`
* `PONTIS_VIEW_LINE_NUMBERS`
* `PONTIS_VIEW_LINE_ENDING_VISIBILITY`

---

## Custom Assets

Additional themes and syntax definitions can be placed here:

* `~/.config/pontis/themes/`
* `~/.config/pontis/syntaxes/`

Notes:

* `themes/` accepts Sublime Text / `syntect` compatible themes
* `syntaxes/` accepts Sublime Text / `syntect` compatible syntax definitions
* If these directories do not exist, `pontis` uses only built-in assets
* Themes placed in `themes/` can be selected with `theme = "..."`

---

## Examples

### Ignore whitespace-only changes

```toml
[compare]
whitespace = "ignore"
inline_diff = true
```

In that case whitespace-only differences are ignored, while inline diff still highlights non-whitespace changes.

### Show line ending markers

```toml
[view]
line_ending_visibility = "all"
```

### Keep backups on save

```toml
[save]
create_backup = true
```

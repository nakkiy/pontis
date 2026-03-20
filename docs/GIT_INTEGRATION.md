# pontis Git Integration Guide

This document covers `pontis` Git comparison modes, `git difftool` integration, and `lazygit` integration.

---

## 1. Git Modes

### working tree vs HEAD

```bash
pontis git
pontis git --repo /path/to/repo
```

### index vs HEAD

```bash
pontis git --staged
pontis git --staged --repo /path/to/repo
```

### working tree vs revision

```bash
pontis git --rev <rev>
pontis git --rev <rev> --repo /path/to/repo
```

### index vs revision

```bash
pontis git --rev <rev> --staged
pontis git --rev <rev> --staged --repo /path/to/repo
```

### revision pair

```bash
pontis git --diff <rev1> <rev2>
pontis git --diff <rev1> <rev2> --repo /path/to/repo
```

---

## 2. Write Rules

Write permissions depend on the comparison mode.

* `pontis git` / `pontis git --rev`: left is read-only, right is writable
* `pontis git --staged`: both sides are read-only
* `pontis git --diff`: both sides are read-only

Notes:

* Merge / save / editor actions are disabled for read-only sides
* `$EDITOR` changes are first applied back into memory, then saved explicitly with `s` / `S`

---

## 3. `git difftool --dir-diff` Integration

You can keep Git in charge of temporary directory creation and still use `pontis`.

### One-time setup

```bash
git config --global diff.tool pontis
git config --global difftool.prompt false
git config --global difftool.pontis.cmd \
  'pontis git --repo "$PWD" \
    --diff "$PONTIS_GIT_DIFFTOOL_LEFT_REV" "$PONTIS_GIT_DIFFTOOL_RIGHT_REV" \
    --difftool-left-dir "$LOCAL" \
    --difftool-right-dir "$REMOTE"'
```

### Example

```bash
REV1=HEAD~1
REV2=HEAD

PONTIS_GIT_DIFFTOOL_LEFT_REV="$REV1" \
PONTIS_GIT_DIFFTOOL_RIGHT_REV="$REV2" \
git difftool --tool pontis --dir-diff "$REV1" "$REV2"
```

In this mode:

* Git creates the temporary directories
* Git comparison metadata is reused for initial `A/D/R`
* `pontis` still computes final `M/=` and hunks

---

## 4. Helper Function

If you use this often, a small helper is easier to type:

```bash
git-pontis-diff() {
  local rev1="$1"
  local rev2="$2"
  PONTIS_GIT_DIFFTOOL_LEFT_REV="$rev1" \
  PONTIS_GIT_DIFFTOOL_RIGHT_REV="$rev2" \
  git difftool --tool pontis --dir-diff "$rev1" "$rev2"
}
```

Example:

```bash
git-pontis-diff HEAD~1 HEAD
```

---

## 5. `lazygit` Integration

Example `~/.config/lazygit/config.yml`:

```yaml
customCommands:
  - key: "D"
    context: "commits"
    description: "Compare the selected commit and its parent with pontis"
    command: >
      PONTIS_GIT_DIFFTOOL_LEFT_REV={{.SelectedLocalCommit.Hash}}^
      PONTIS_GIT_DIFFTOOL_RIGHT_REV={{.SelectedLocalCommit.Hash}}
      git difftool --tool pontis --dir-diff
      {{.SelectedLocalCommit.Hash}}^ {{.SelectedLocalCommit.Hash}}
    output: terminal

  - key: "R"
    context: "commits"
    description: "Compare the selected range with pontis"
    command: >
      PONTIS_GIT_DIFFTOOL_LEFT_REV={{.SelectedCommitRange.From}}
      PONTIS_GIT_DIFFTOOL_RIGHT_REV={{.SelectedCommitRange.To}}
      git difftool --tool pontis --dir-diff
      {{.SelectedCommitRange.From}} {{.SelectedCommitRange.To}}
    output: terminal
```

---

## 6. How Rename / Move Is Shown

In Git comparisons, `pontis` treats rename and move as `R`.

Notes:

* `R` is a file-list display / filter / path-pairing concept
* `rename only` is still `R`
* `rename + modify` is also `R`, with normal content diff shown
* Merge moves content only; it does not automatically perform the rename or move itself

---

## 7. Notes

* `git --staged` and `git --diff` are read-only comparisons
* Binary files are not rendered as text diffs
* Binary files do not support hunk merge

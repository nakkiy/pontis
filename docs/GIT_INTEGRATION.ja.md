# pontis Git 連携ガイド

この文書は、`pontis` の Git 比較モード、`git difftool` 連携、`lazygit` 連携をまとめたものです。

---

## 1. Git モード

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

## 2. 書き込み制約

Git 比較では比較対象によって編集可否が異なります。

* `pontis git` / `pontis git --rev`: 左は read-only、右は writable
* `pontis git --staged`: 左右とも read-only
* `pontis git --diff`: 左右とも read-only

補足:

* read-only 側では merge / save / editor 連携は無効です
* `$EDITOR` 連携の結果はまずメモリに取り込まれ、保存は `s` / `S` で明示的に行います

---

## 3. `git difftool --dir-diff` 連携

`git difftool --dir-diff` を使う場合でも、一時ディレクトリ作成は Git に任せたまま `pontis` を使えます。

### 一度だけ設定

```bash
git config --global diff.tool pontis
git config --global difftool.prompt false
git config --global difftool.pontis.cmd \
  'pontis git --repo "$PWD" \
    --diff "$PONTIS_GIT_DIFFTOOL_LEFT_REV" "$PONTIS_GIT_DIFFTOOL_RIGHT_REV" \
    --difftool-left-dir "$LOCAL" \
    --difftool-right-dir "$REMOTE"'
```

### 実行例

```bash
REV1=HEAD~1
REV2=HEAD

PONTIS_GIT_DIFFTOOL_LEFT_REV="$REV1" \
PONTIS_GIT_DIFFTOOL_RIGHT_REV="$REV2" \
git difftool --tool pontis --dir-diff "$REV1" "$REV2"
```

この経路では、

* 一時ディレクトリ作成は Git が担当
* file list の `A/D/R` は Git の比較文脈を使う
* `M/=` 判定と hunk 計算は `pontis` が担当

---

## 4. helper 関数

頻繁に使うなら関数を用意しておくと短く呼べます。

```bash
git-pontis-diff() {
  local rev1="$1"
  local rev2="$2"
  PONTIS_GIT_DIFFTOOL_LEFT_REV="$rev1" \
  PONTIS_GIT_DIFFTOOL_RIGHT_REV="$rev2" \
  git difftool --tool pontis --dir-diff "$rev1" "$rev2"
}
```

使用例:

```bash
git-pontis-diff HEAD~1 HEAD
```

---

## 5. lazygit 連携

`~/.config/lazygit/config.yml` の例:

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

## 6. rename / move の見え方

`pontis` は Git 比較時に、移動やリネームを `R` として扱います。

補足:

* `R` は file list 上の表示・filter・対応付けの概念です
* `rename only` でも `R` です
* `rename + modify` でも `R` のまま内容 diff を表示します
* merge は内容だけを動かし、rename / move 自体を自動反映はしません

---

## 7. 注意点

* `git --staged` / `git --diff` は read-only 比較です
* `l` による再読込は `pontis git` と `pontis git --rev` で使えますが、`git --staged`、`git --diff`、`git difftool` 連携では使えません
* 未保存変更がある間は再読込できません
* binary ファイルはテキスト diff 表示されません
* binary への hunk merge はできません

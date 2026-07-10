---
name: bump-and-release
description: 負責升級專案版本（patch、minor 或 major），更新所有設定檔與原始碼中的版本號，並協調發布流程、Git 提交、遠端 CI 成功閘門、Tag 推送與 GitHub Release 繁體中文發行說明更新。
---

# `bump-and-release` 技能說明

本技能旨在指導 AI 代理人如何為 `ask-bridge` 專案執行正確的版本號提升（Version Bumping）、發布（Release）準備工作，以及 GitHub Release 建立後的繁體中文發行說明維護。

## 🎯 適用場景
當使用者要求：
- 「Bump version / Bump patch version / Bump minor version」
- 「升級版本號 / 釋出新版本 / 準備 Release」
- 「Bump and release」

請務必啟用並遵守本技能所述之標準作業程序，確保所有平台的設定與安裝腳本中的版本資訊保持一致。

---

## 📋 待更新之檔案清單

版本號必須在下列檔案中同步更新：

| 檔案路徑 | 欄位或變數格式 | 說明 |
| :--- | :--- | :--- |
| **`Cargo.toml`** | `version = "X.Y.Z"` | Rust 專案定義檔。 |
| **`package.json`** | `"version": "X.Y.Z",` | NPM 套件定義檔。 |
| **`src/main.rs`** | `#[command(version = "X.Y.Z")]` | CLI 命令列工具的內建版本顯示（Clap 屬性）。 |
| **`install.ps1`** | `$Version = "X.Y.Z"` | Windows PowerShell 安裝腳本中的下載版本。 |
| **`install.sh`** | `VERSION="X.Y.Z"` | Linux/macOS Shell 安裝腳本中的下載版本。 |
| **`scripts/ask.sh`** | `VERSION="X.Y.Z"` | 專案輔助腳本的版本號。 |

> [!IMPORTANT]
> 以上 6 個檔案的版本號必須完全一致。更新後，**必須執行編譯驗證**以自動更新 **`Cargo.lock`**。

---

## 🔄 標準作業程序 (SOP)

### 第一步：分析升級類型
1. **Patch 升級** (`X.Y.Z` 變為 `X.Y.(Z+1)`)：適用於 Bug 修正、極小功能微調。
2. **Minor 升級** (`X.Y.Z` 變為 `X.(Y+1).0`)：適用於新增功能（向下相容）。
3. **Major 升級** (`X.Y.Z` 變為 `(X+1).0.0`)：適用於有重大破壞性變更（Breaking Changes）的更新。

### 第二步：修改版本號
使用適當的程式碼編輯工具，將上述 **6 個檔案** 的舊版本號替換為新版本號。

### 第三步：執行驗證與更新 Lockfile
修改完設定檔後，必須執行 Rust 的程式檢查，這會自動同步更新 `Cargo.lock`：
```powershell
cargo check
```
（可選擇執行 `cargo build --release` 確保 release 版本能正常建置）

若要驗證 npm 的相關測試，可於專案目錄執行：
```powershell
npm test
```

### 第四步：Git 提交並推送主分支
遵照專案的 Git 規範，在提交時應將所有版本號相關的修改合併為一個單一提交，並採用 **Conventional Commits 1.0.0** 規範，日誌應提供完整的繁體中文（zh-tw）說明。

此步驟只允許推送 `main`，不得建立或推送 Tag。推送前先記錄待發布 commit SHA，後續 CI 查詢、Tag 建立與一致性檢查都必須使用此不可變 SHA，不得改用等待期間可能變動的 `HEAD`。

#### 執行範例：
```bash
set -euo pipefail

# 1. 提交所有版本相關變更，且確認目前位於 main
[ "$(git branch --show-current)" = "main" ] || {
  echo "目前不在 main；禁止發布。" >&2
  exit 1
}

git add .

commit_msg_file="$(mktemp -t codex-commit-message)"
cat > "$commit_msg_file" <<'EOF'
chore(release): bump version to X.Y.Z

此提交將專案各平台之版本與設定檔同步提升至 X.Y.Z：

- 更新 Cargo.toml 與 package.json 套件版本號。
- 更新 src/main.rs 中 Clap 內建 CLI 版本資訊。
- 同步更新 install.ps1、install.sh 與 scripts/ask.sh 腳本中的發布版本變數。
- 自動同步更新 Cargo.lock。
EOF

git commit -F "$commit_msg_file"
release_commit="$(git rev-parse HEAD)"
git push origin main
```

### 第五步：等待並確認遠端 CI 成功
本機驗證通過後，仍必須等待 GitHub Actions 的 `.github/workflows/ci.yml` 對同一個 release commit 完成。只有符合以下全部條件，才能進入 Tag 步驟：

1. Workflow 必須是 `ci.yml`，事件必須是 `push`。
2. `headSha` 必須等於第四步保存的 `release_commit`。
3. `status` 必須是 `completed`，`conclusion` 必須是 `success`。
4. CI 完成後，`origin/main` 仍必須精確指向 `release_commit`。

#### CI 品質閘門範例：
```bash
repo="doggy8088/ask-bridge"

# 先確認 main push 已更新到待發布 commit
git fetch origin main --quiet
if [ "$(git rev-parse refs/remotes/origin/main)" != "$release_commit" ]; then
  echo "遠端 main 與待發布 commit 不一致；禁止建立或推送 Tag。" >&2
  exit 1
fi

# GitHub Actions 建立 run 可能有短暫延遲；最多等待 5 分鐘讓 run 出現
ci_run_id=""
for _ in $(seq 1 30); do
  ci_run_id="$(gh run list \
    --repo "$repo" \
    --workflow ci.yml \
    --event push \
    --commit "$release_commit" \
    --limit 1 \
    --json databaseId \
    --jq '.[0].databaseId // empty')"
  [ -n "$ci_run_id" ] && break
  sleep 10
done

if [ -z "$ci_run_id" ]; then
  echo "逾時仍找不到 commit $release_commit 的 main push CI；禁止建立或推送 Tag。" >&2
  exit 1
fi

# failure、cancelled、timed_out 或其他非 success 結果都必須停止
if ! gh run watch "$ci_run_id" --repo "$repo" --exit-status --interval 10; then
  gh run view "$ci_run_id" --repo "$repo"
  echo "CI run $ci_run_id 未成功；禁止建立或推送 Tag。" >&2
  exit 1
fi

# watch 結束後再次驗證不可變後置條件，避免誤用其他 run
ci_state="$(gh run view "$ci_run_id" \
  --repo "$repo" \
  --json headSha,event,status,conclusion \
  --jq '[.headSha,.event,.status,.conclusion] | @tsv')"
expected_ci_state="$(printf '%s\tpush\tcompleted\tsuccess' "$release_commit")"
if [ "$ci_state" != "$expected_ci_state" ]; then
  echo "CI 後置條件不符：$ci_state；禁止建立或推送 Tag。" >&2
  exit 1
fi

# 等待期間 main 可能已前進；正式發布只允許目前 main 的最新 commit
git fetch origin main --quiet
if [ "$(git rev-parse refs/remotes/origin/main)" != "$release_commit" ]; then
  echo "CI 完成後遠端 main 已變更；應以新的 main HEAD 重新執行發布流程。" >&2
  exit 1
fi
```

> [!CAUTION]
> 找不到 CI run、等待逾時、`gh auth`／GitHub API／網路錯誤，或任何非 `success` 結果時，一律停止。不得以本機測試、PR CI、手動 workflow run 或舊 commit 的成功結果取代此閘門。修正問題若產生新 commit，必須更新 `release_commit`、重新推送並等待新 commit 的 CI。

### 第六步：建立並推送 Tag
CI 品質閘門成功後，才可建立對應的 **Git Tag**（格式為 `vX.Y.Z`）。Tag 必須明確指向已通過 CI 的 `release_commit`，不得依賴目前 `HEAD`，也不得覆寫或強制推送既有 Tag。

#### 執行範例：
```bash
tag="vX.Y.Z"

# 本機或遠端已有同名 Tag 時停止，先釐清既有 release 狀態
if git show-ref --verify --quiet "refs/tags/$tag"; then
  echo "本機 Tag $tag 已存在；禁止覆寫。" >&2
  exit 1
fi

if git ls-remote --exit-code --tags origin "refs/tags/$tag" >/dev/null 2>&1; then
  echo "遠端 Tag $tag 已存在；禁止覆寫。" >&2
  exit 1
fi

# 建立帶有註解的 Tag，明確鎖定已通過 CI 的 commit
git tag -a "$tag" "$release_commit" -m "Release $tag"

# 推送 Tag 後會觸發 Release，Release 發布後再觸發 Publish npm
git push origin "refs/tags/$tag"
```

### 第七步：更新 GitHub Release 繁體中文發行說明
GitHub Release 建立完成後，必須立即補上繁體中文（zh-tw）發行說明。不要只保留 GitHub 自動產生的 `Full Changelog` 連結。

#### 資料蒐集
1. 使用 `gh release view vX.Y.Z --repo doggy8088/ask-bridge --json tagName,body,url,publishedAt` 確認 release 已建立。
2. 使用 `git log --reverse --pretty=format:'%h%x09%ad%x09%s' --date=short <previous-tag>..vX.Y.Z` 查看此版本 commit。若是首版 release，前一個 tag 不存在，改用 `git log --reverse --pretty=format:'%h%x09%ad%x09%s' --date=short vX.Y.Z`。
3. 使用 `git diff --stat <previous-tag>..vX.Y.Z` 與必要的 `git diff <previous-tag>..vX.Y.Z -- <path>` 確認實際影響範圍。若是首版 release，改用：
   - `git diff --stat --root vX.Y.Z`
   - `git diff --root vX.Y.Z -- <path>`
4. 若 `CHANGELOG.md` 已有該版本內容，使用 `git show vX.Y.Z:CHANGELOG.md` 交叉比對，但仍需以 commit 與 diff 驗證，不可只改寫 changelog。

#### 發行說明格式
發行說明必須使用 Markdown 與繁體中文（zh-tw），用詞保持台灣用語。建議包含以下區塊，並依實際變更刪減：

```markdown
## 發行重點

用 1 到 2 句說明此版本最重要的使用者可見變更。

## 新增

- 條列新增功能。

## 修正

- 條列錯誤修正或相容性修正。

## 文件與網站

- 條列文件、網站、README 或中繼資料更新。

## 測試

- 條列新增或執行過的驗證。

## 注意事項

- 條列安裝、相容性或遷移注意事項。

## 相關連結

- 完整變更紀錄: https://github.com/doggy8088/ask-bridge/compare/<previous-tag>...vX.Y.Z
```

首版 release 的完整變更紀錄連結可使用：

```markdown
- 完整變更紀錄: https://github.com/doggy8088/ask-bridge/commits/vX.Y.Z
```

#### 寫入 GitHub Release
將發行說明寫入亂數暫存檔，確認 UTF-8 純文字後使用 `gh release edit` 更新：

```bash
release_notes_file="$(mktemp -t codex-release-notes)"
cat > "$release_notes_file" <<'EOF'
## 發行重點

本版...

## 相關連結

- 完整變更紀錄: https://github.com/doggy8088/ask-bridge/compare/<previous-tag>...vX.Y.Z
EOF

gh release edit vX.Y.Z --repo doggy8088/ask-bridge --notes-file "$release_notes_file"
gh release view vX.Y.Z --repo doggy8088/ask-bridge --json tagName,body,url
```

> [!IMPORTANT]
> 發行說明必須描述已由 commit、diff、`CHANGELOG.md` 或 release asset 狀態驗證過的事實。若查無足夠資料，必須明確寫出不確定範圍，禁止臆測。

---

## 💡 提示與訣竅

> [!TIP]
> 變更版本號前，建議先檢查舊版本號（例如 `0.1.2`）是否在其他檔案中出現：
> - macOS / Linux：
>   - `rg -n "0\.1\.2" .`
>   - 若無 `rg`，改用：`grep -RIn "0.1.2" .`
> - Windows PowerShell：
>   - `rg -n "0\.1\.2" .`
>   - 若無 `rg`，改用：`Get-ChildItem -Recurse -File | Select-String -Pattern "0.1.2" -SimpleMatch`

> [!CAUTION]
> 絕對不要只修改 `Cargo.toml` 而忽略了安裝腳本中的 `$Version`，這會導致使用者透過 `curl` 或 `iwr` 下載安裝時抓取到錯誤的版本。

---
name: bump-and-release
description: 負責升級專案版本（patch、minor 或 major），更新所有設定檔與原始碼中的版本號，並協調發布流程、Git 提交、Tag 推送與 GitHub Release 繁體中文發行說明更新。
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

### 第四步：Git 提交、打 Tag 與推送
遵照專案的 Git 規範，在提交時應將所有版本號相關的修改合併為一個單一提交，並採用 **Conventional Commits 1.0.0** 規範，日誌應提供完整的繁體中文（zh-tw）說明。

推送主分支後，必須為該版本建立對應的 **Git Tag**（格式為 `vX.Y.Z`），並將 Tag 獨立推送到遠端倉庫以完成正式釋出。

#### 執行範例：
```bash
# 1. 提交所有版本相關變更
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
git push

# 2. 建立帶有註解的 Git Tag (以 vX.Y.Z 為例)
git tag -a vX.Y.Z -m "Release vX.Y.Z"

# 3. 推送 Tag 到遠端（將觸發 GitHub Releases 流程）
git push origin vX.Y.Z
```

### 第五步：更新 GitHub Release 繁體中文發行說明
GitHub Release 建立完成後，必須立即補上繁體中文（zh-tw）發行說明。不要只保留 GitHub 自動產生的 `Full Changelog` 連結。

#### 資料蒐集
1. 使用 `gh release view vX.Y.Z --repo doggy8088/ask-bridge --json tagName,body,url,publishedAt` 確認 release 已建立。
2. 使用 `git log --reverse --pretty=format:'%h%x09%ad%x09%s' --date=short <previous-tag>..vX.Y.Z` 查看此版本 commit。若是首版 release，範圍改用 `vX.Y.Z`。
3. 使用 `git diff --stat <previous-tag>..vX.Y.Z` 與必要的 `git diff <previous-tag>..vX.Y.Z -- <path>` 確認實際影響範圍。
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
> 變更版本號前，建議使用 `grep_search` 搜尋當前的舊版本號（例如 `0.1.2`），以確認是否有其他遺漏未提及的檔案需要同步更新。

> [!CAUTION]
> 絕對不要只修改 `Cargo.toml` 而忽略了安裝腳本中的 `$Version`，這會導致使用者透過 `curl` 或 `iwr` 下載安裝時抓取到錯誤的版本。

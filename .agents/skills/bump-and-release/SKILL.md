---
name: bump-and-release
description: 負責升級專案版本（patch、minor 或 major），更新所有設定檔與原始碼中的版本號，並協調發布流程與 Git 提交。
---

# `bump-and-release` 技能說明

本技能旨在指導 AI 代理人如何為 `ask-bridge` 專案執行正確的版本號提升（Version Bumping）與發布（Release）準備工作。

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
使用適當的代碼編輯工具（如 `replace_file_content`），將上述 **6 個檔案** 的舊版本號替換為新版本號。

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

### 第四步：Git 提交與推送
遵照專案的 Git 規範，在提交時應將所有版本號相關的修改合併為一個單一提交，並採用 **Conventional Commits 1.0.0** 規範，日誌應提供完整的繁體中文（zh-tw）說明。

#### Commit 範例：
```bash
git add .
git commit -m "chore: bump version to X.Y.Z" -m "此提交將專案各平台之版本與設定檔同步提升至 X.Y.Z：

- 更新 Cargo.toml 與 package.json 套件版本號。
- 更新 src/main.rs 中 Clap 內建 CLI 版本資訊。
- 同步更新 install.ps1、install.sh 與 scripts/ask.sh 腳本中的發布版本變數。
- 自動同步更新 Cargo.lock。"
git push
```

---

## 💡 提示與訣竅

> [!TIP]
> 變更版本號前，建議使用 `grep_search` 搜尋當前的舊版本號（例如 `0.1.2`），以確認是否有其他遺漏未提及的檔案需要同步更新。

> [!CAUTION]
> 絕對不要只修改 `Cargo.toml` 而忽略了安裝腳本中的 `$Version`，這會導致使用者透過 `curl` 或 `iwr` 下載安裝時抓取到錯誤的版本。

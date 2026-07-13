---
name: ask-bridge
description: "完整使用 ask-bridge CLI 的 Agent Skill。使用 ask-bridge 將低風險、探索性 AI 研究、摘要、文件分析、程式片段分析、錯誤訊息整理、方案比較、初稿產出或可委派的背景調查交給 ChatGPT、Gemini 或 Claude 網站。當 Codex 需要透過本機 ask-bridge 命令呼叫網站型 AI、利用 ChatGPT/Gemini/Claude 網頁額度、附加檔案或圖片、切換模型、取回回覆、下載生成圖片、管理瀏覽器 session、或查詢 ask-bridge 所有參數與子命令用法時使用。"
---

# Ask Bridge

## 核心原則

使用 `ask-bridge` 把低風險、探索性、可委派的 AI 任務交給 ChatGPT、Gemini 或 Claude 網站處理，再將回覆作為本機工作流程的參考輸入。不要把 provider 回覆視為事實來源、測試結果或已完成的程式碼變更。

優先把主要 Coding Agent 保留給下列工作：讀取專案脈絡、修改檔案、執行測試、驗證行為、整合結論。把 `ask-bridge` 用於背景研究、摘要、候選方案、初稿與輔助分析。

## 執行命令名稱

安裝後優先使用 `ask-bridge`：

```sh
ask-bridge --help
```

本專案產出的 Rust binary 叫 `ask-bridge`，release 產物通常位於 `target/release/ask-bridge`。在專案內測試尚未安裝的版本時，可使用：

```sh
cargo run -- --help
target/release/ask-bridge --help
```

本 Skill 下方範例一律以安裝後的 `ask-bridge` 命令表示。

`ask` 只是向後相容 alias。除非使用者明確要求 alias，文件、範例與自動化命令都優先使用 `ask-bridge`。

## 使用前檢查

先確認 `ask-bridge` 可用：

```sh
command -v ask-bridge
ask-bridge -v
```

若找不到 `ask-bridge`，在 ask-bridge 專案中可先使用既有安裝流程、`cargo run --`，或建置後的 `target/release/ask-bridge`；不要臆測使用者環境已完成設定。

## 全域設定檔

`ask-bridge` 會讀取全域設定檔：

```text
~/.config/ask-bridge/config.json
```

可用格式：

```json
{
  "provider": "gemini"
}
```

或：

```json
{
  "provider": "chatgpt"
}
```

或：

```json
{
  "provider": "claude"
}
```

provider 優先序：

1. CLI `--provider chatgpt|gemini|claude`
2. `~/.config/ask-bridge/config.json` 的 `provider`
3. 內建預設 `chatgpt`

若需要替使用者設定預設 provider，可建立設定檔：

```sh
ask-bridge config --provider gemini
ask-bridge config --provider claude
```

若要改回 ChatGPT：

```sh
ask-bridge config --provider chatgpt
```

使用 `ask-bridge config` 可查看目前設定：

```sh
ask-bridge config
```

若任務需要單次覆蓋全域設定，直接使用 `--provider`，不要修改設定檔：

```sh
ask-bridge --provider chatgpt '請摘要這段內容。'
```

首次使用或登入失效時，`ask-bridge` 可能需要瀏覽器互動登入。只有在任務允許互動式登入時才執行：

```sh
ask-bridge login
ask-bridge --provider gemini login
ask-bridge --provider claude login
ask-bridge login --provider gemini
```

若目前任務不適合中斷等待登入，回報需要使用者完成登入，不要反覆重試。

## 委派決策

適合使用 `ask-bridge`：

- 摘要長文件、錯誤訊息、測試輸出、issue 討論或研究筆記。
- 請外部 AI 對程式片段、設計方案或規格草稿做初步分析。
- 產生候選實作策略、檢查清單、測試案例構想或文件初稿。
- 比較多個方案的優缺點，但由本機 agent 保留最終判斷。
- 將不需要直接修改專案檔案的工作移出主要 agent 流程。

避免使用 `ask-bridge`：

- 需要讀取或傳送密鑰、憑證、token、個資、內部機密或使用者未授權的內容。
- 需要可驗證最新資訊、法規、價格、版本、新聞或官方規格時，把 `ask-bridge` 當成唯一資料來源。
- 需要直接修改檔案、執行測試、操作 git、發佈或部署。
- 任務要求嚴格可重現、可稽核或不可容忍 provider 幻覺。

## 快速語法

基本語法：

```sh
ask-bridge [OPTIONS] [PROMPT] [COMMAND]
```

常用範例：

```sh
ask-bridge '請摘要下列錯誤訊息，列出可能原因與下一步檢查。'
ask-bridge --provider gemini '請比較這三個實作方向的風險與取捨。'
cat report.md | ask-bridge '請摘要這份文件，列出待辦。'
ask-bridge '請摘要這份規格文件。' --file docs/spec.md -o /tmp/spec-summary.md
ask-bridge '請描述這張截圖中的 UI 問題。' --image screenshot.png
ask-bridge '請根據這份 prompt 產生圖片。' -i /tmp/generated-images/
```

若同時提供 prompt argument 與 stdin，`ask-bridge` 會組合為：

```text
prompt + "\n\n" + stdin
```

## 參數速查

| 參數 | 用途 | 用法重點 |
|---|---|---|
| `[PROMPT]` | 要送給 provider 的文字 prompt | 可省略；若 stdin 有內容則使用 stdin；若兩者都有，會以兩個換行串接 |
| `-p`, `--provider <PROVIDER>` | 選擇 provider | 可用 `chatgpt`、`gemini` 或 `claude`；此為 global option，可放在子命令前後；優先權高於全域設定檔 |
| `--headless[=<HEADLESS>]` | 控制 Chrome 是否 headless | 預設 `true`；要顯示瀏覽器請用 `--headless=false`；不要寫成 `--headless false` |
| `--new` | 開啟全新 provider 對話 | 會開新分頁並清理同 provider 舊分頁；用於隔離上下文 |
| `-v`, `-V`, `--version` | 顯示版本 | `-V` 是原始碼中定義的短別名；文件與一般操作優先用 `-v` 或 `--version` |
| `--verbose` | 顯示瀏覽器自動化流程 | 用於診斷 provider UI、登入、上傳、模型切換或等待回覆問題 |
| `-o`, `--output <FILE>` | 將最終 Markdown 回覆寫入檔案 | 同時仍會在終端機輸出渲染結果；適合保留研究紀錄 |
| `-i`, `--image-output <IMAGE_PATH>` | 下載 provider 回覆中的生成圖片 | 可指定資料夾或檔案路徑；可搭配一般 prompt、`get` 或 `open <url>` |
| `--image <IMAGE_FILE>` | 附加圖片檔，可重複指定 | 支援 ChatGPT 與 Claude；搭配 Gemini 會失敗 |
| `--file <FILE>` | 附加文件檔，可重複指定 | 支援 PDF、Word、Excel、PowerPoint、純文字、Markdown、CSV、JSON、程式碼等；ChatGPT、Gemini 與 Claude 都可用 |
| `--timeout <SECONDS>` | 最大等待回應時間（秒） | 預設為 `300` 秒，最小值為 `1`；provider 回應較慢時可適度調高 |
| `--model <MODEL>` | 送出 prompt 前切換模型 | 比對不分大小寫與標點；模型名稱取決於 provider UI 與帳號權限 |
| `-h`, `--help` | 顯示 help | 可用 `ask-bridge --help` 或 `ask-bridge help <COMMAND>` |
| `config` | 設定或顯示全域預設 provider | 使用 `ask-bridge config --provider <chatgpt|gemini|claude>` |

## Provider 選擇

預設使用 ChatGPT：

```sh
ask-bridge '請摘要下列錯誤訊息，列出可能原因與下一步檢查。'
ask-bridge --provider chatgpt '請分析這段程式碼的風險。'
ask-bridge -p chatgpt '請整理這份文件的待辦。'
```

使用 Gemini 或 Claude 時明確指定 provider：

```sh
ask-bridge --provider gemini '請比較這三個實作方向的風險與取捨。'
ask-bridge -p gemini '請摘要這份文件。' --file notes.md
ask-bridge --provider claude '請初步分析這段程式碼的風險。'
ask-bridge -p claude '請摘要這份文件。' --file notes.md
```

選擇原則：

- 未指定 `--provider` 時，先依全域設定檔選擇 provider；設定檔不存在時使用 ChatGPT。
- 使用 ChatGPT 作為未設定時的預設 provider。
- 使用 Gemini 做替代觀點、快速摘要或使用者明確要求 Gemini 時。
- 使用 Claude 做程式碼分析、長文摘要、替代觀點或使用者明確要求 Claude 時；Claude 也支援 `--image` 圖片輸入。
- 若 provider 失敗，可在不增加風險的情況下改用另一個 provider 一次。
- 不要硬編不存在的模型名稱；只有使用者指定或專案文件明確列出時才使用 `--model`。

## Prompt 組裝

讓委派 prompt 包含明確輸出契約：

```text
你是協助主要 Coding Agent 的研究助手。
目標：<要完成的分析或摘要>
背景：<必要上下文>
輸入資料：<貼上內容或說明附件>
請輸出：
1. 直接結論
2. 依據與不確定處
3. 可執行的下一步
限制：不要聲稱已修改本機檔案；不確定時請明確標示。
```

保持 prompt 聚焦。大型任務先請 provider 產生摘要或候選清單，再由本機 agent 判斷是否需要第二輪委派。

## 傳入資料

對短文字使用 argument：

```sh
ask-bridge '請用 5 點摘要這段錯誤訊息，並標示最可能的根因。'
```

對程式片段或命令輸出使用 stdin：

```sh
cargo test 2>&1 | ask-bridge '請摘要測試失敗重點，列出可能要看的檔案與函式。'
cat src/main.rs | ask-bridge '請初步檢查這段 Rust 程式碼的錯誤處理與風險。'
```

同時使用 prompt 與 stdin：

```sh
cat docs/spec.md | ask-bridge '請根據下列規格產生實作檢查清單。'
```

對完整文件、二進位文件或多檔案使用 `--file`：

```sh
ask-bridge '請摘要這份規格文件，列出實作需求與待釐清問題。' --file docs/spec.md
ask-bridge '請比較這兩份文件的差異。' --file old.md --file new.md
ask-bridge --provider gemini '請摘要這份 PDF。' --file report.pdf
```

對圖片使用 `--image`，目前支援 ChatGPT 與 Claude：

```sh
ask-bridge '請描述這張截圖中的 UI 問題，並列出可能的 CSS 原因。' --image screenshot.png
ask-bridge '請比較這兩張圖的差異。' --image before.png --image after.png
ask-bridge --provider claude '請描述這張截圖中的 UI 問題。' --image screenshot.png
```

同時附加圖片與文件時，使用 ChatGPT 或 Claude：

```sh
ask-bridge '請對照設計圖與規格文件，列出不一致處。' --image design.png --file spec.md
```

## 輸出與下載

將 Markdown 回覆寫入檔案：

```sh
ask-bridge '請整理這份輸入的重點與待辦。' --file notes.md --output /tmp/ask-notes.md
ask-bridge '請整理這份輸入的重點與待辦。' --file notes.md -o /tmp/ask-notes.md
```

下載 provider 回覆中的生成圖片：

```sh
ask-bridge '請產生一張產品概念圖。' --image-output /tmp/ask-images/
ask-bridge '請產生一張產品概念圖。' -i /tmp/product-concept.png
```

若只需要保存文字研究結果，也可用 shell redirect：

```sh
ask-bridge '請整理這份輸入的重點與待辦。' --file notes.md > /tmp/ask-bridge-notes.md
```

## 模型切換

使用 `--model` 在送出 prompt 前切換模型：

```sh
ask-bridge '用幾句話介紹 Rust。' --model GPT-5.4
ask-bridge '證明這個數學問題。' --model o3
ask-bridge '快速翻譯這段話。' --model 即時
ask-bridge --provider gemini '用幾句話介紹 Rust。' --model '3.5 Flash'
ask-bridge --provider gemini '用幾句話介紹 Rust。' --model '3.1 Pro'
ask-bridge --provider claude '用幾句話介紹 Rust。' --model Sonnet
ask-bridge --provider claude '證明這個數學問題。' --model Opus
```

模型比對不分大小寫與標點符號。若切換失敗，移除 `--model` 或改用 provider 預設模型，不要猜測替代模型名稱。

## ChatGPT Agent 提及語法

使用 ChatGPT 時，prompt 可採用 `@Agent名稱 prompt正文` 格式。
`ask-bridge` 會先輸入 Agent mention、等待候選選單出現後按 `Tab` 選取第一個候選項目，再貼上正文。

### 語法與限制

- **格式**：`@Agent名稱 prompt正文`
- **名稱限制**：Agent 名稱必須由 1 至 10 個非空白字元組成。
- **正文限制**：Agent 名稱後必須至少有一個空白，且正文不可為空。
- **Provider 限制**：此特殊處理只適用於 ChatGPT；Gemini 與 Claude 會將它作為一般文字 prompt。
- **Fallback**：格式不符合時，不會觸發 Agent mention，而會按照一般 prompt 處理。

### 使用範例

```sh
ask-bridge '@研究助手 請摘要這份規格並列出風險。' \
  --file docs/spec.md

ask-bridge --provider chatgpt \
  '@程式審查 請檢查這份程式碼的風險。' \
  --file src/main.rs
```

## 對話與瀏覽器控制

需要隔離上下文時使用 `--new`：

```sh
ask-bridge '請只根據本次輸入分析，不要沿用既有對話脈絡。' --new
```

一般提問預設 `--headless=true`。需要觀察 Chrome 操作時使用：

```sh
ask-bridge '請回覆 ok' --headless=false
ask-bridge '請回覆 ok' --verbose --headless=false
```

關閉 ask-bridge 管理的 Chrome instance：

```sh
ask-bridge close
```

`close` 只應關閉 ask-bridge 使用的 debug profile 與 debug port instance。若 port 被非 ask-bridge Chrome 程序占用，工具應回報錯誤而不是關閉它。

## 子命令速查

公開子命令：

| 子命令 | 用途 | 範例 |
|---|---|---|
| `login` | 開啟 provider 並等待使用者手動登入 | `ask-bridge login`、`ask-bridge --provider gemini login`、`ask-bridge --provider claude login` |
| `close` | 關閉 ask-bridge 管理的 Chrome instance | `ask-bridge close` |
| `config` | 顯示或設定全域預設 provider | `ask-bridge config`、`ask-bridge config --provider gemini` |
| `update` | 使用官方安裝流程更新 ask-bridge | `ask-bridge update` |
| `help` | 顯示 help | `ask-bridge help`、`ask-bridge help login` |

隱藏或維護用子命令：

| 子命令 | 用途 | 範例 |
|---|---|---|
| `open [url]` | 不帶 URL 時開啟 provider；帶 URL 時開啟該對話並複製最新回覆 | `ask-bridge open`、`ask-bridge open 'https://chatgpt.com/c/...'` |
| `get [url]` | 從目前 provider 或指定 URL 取得最新回覆 | `ask-bridge get`、`ask-bridge get 'https://gemini.google.com/app/...' -o /tmp/reply.md` |
| `dump` | 將目前分頁 HTML 寫到 `target/dump.html`，供除錯使用 | `ask-bridge dump --verbose` |
| `screenshot` | 將目前分頁截圖寫到 `target/screenshot.png`，供除錯使用 | `ask-bridge screenshot --headless=false` |

`open`、`get`、`dump`、`screenshot` 可能不出現在一般 `--help` 中。除錯或維護 ask-bridge 自動化流程時才使用隱藏子命令；一般委派任務優先使用 prompt、`--file`、`--image`、`--output` 與 `--image-output`。

## 使用回覆

將回覆視為外部建議：

- 先閱讀並萃取可驗證的部分，再套用到本機工作。
- 對程式建議，仍需讀原始碼、修改檔案並執行測試。
- 對最新資訊或官方規格，必須再用可靠來源查證。
- 對不確定或互相矛盾的結果，保留疑點，不要包裝成確定結論。

## 失敗處理

登入、CAPTCHA、Cloudflare 或 session 失效時，要求使用者完成 `ask-bridge login`，或在允許互動式瀏覽器時執行登入流程。

provider UI 變更或自動化失敗時，使用 `--verbose` 取得診斷資訊；必要時用 `--headless=false` 觀察瀏覽器狀態：

```sh
ask-bridge '請回覆 ok' --verbose
ask-bridge '請回覆 ok' --headless=false
ask-bridge dump --verbose
ask-bridge screenshot --headless=false
```

Gemini 圖片輸入不支援時，改用 ChatGPT 或 Claude，或改以文字描述圖片內容。不要把同一個失敗命令無限制重試。

模型切換失敗時，移除 `--model` 或改用 provider 預設模型，不要猜測替代模型名稱。

沒有 prompt 且沒有子命令時，`ask-bridge` 會顯示 help。若任務需要非互動輸出，優先搭配 `--output` 或 shell redirect，避免只依賴終端機渲染結果。

# 快速開始

本文件說明如何安裝 `ask`，並透過 Chrome 自動操作 ChatGPT。

## 前置需求

- macOS，且已安裝 Cargo。
- Shell 可使用 Node.js 與 `npx`。
- 若希望 `make install` 在缺少 Google Chrome 時自動安裝 Chrome，需先安裝 Homebrew。

不需要安裝全域 `mcp-cli` 執行檔。本專案會透過 Cargo 從 `https://github.com/doggy8088/mcp-cli` 使用 `mcp-cli` 作為 Rust dependency。

## 安裝

執行：

```sh
make install
```

此命令會：

- 在缺少 Google Chrome 時，透過 Homebrew 安裝 Chrome。
- 建置 release binary。
- 建立 `~/.local/bin/ask` symlink，指向 release binary。

請確認 `~/.local/bin` 已加入你的 `PATH`。

## 首次登入

執行：

```sh
ask login
```

Chrome 會使用專屬 profile 開啟，profile 路徑為：

```text
~/.config/ask-chatgpt/chrome-profile
```

在瀏覽器視窗登入 ChatGPT 後，回到終端機按 Enter。

## 提問

執行：

```sh
ask "用一段話解釋 Rust ownership。"
```

一般提問預設會使用 headless Chrome，並把 ChatGPT 回覆串流輸出到終端機。

## 使用可見瀏覽器

執行：

```sh
ask "請示範一個簡短 Markdown 表格。" --headless=false
```

這會在自動化執行期間保持 Chrome 視窗可見。

## 開啟新的 ChatGPT 對話

執行：

```sh
ask "開始一個關於 async Rust 的新主題。" --new
```

`--new` 會開啟新的 ChatGPT 對話，而不是重用既有的 ChatGPT 分頁。

## 透過 pipe 傳入內容

執行：

```sh
cat README.md | ask "摘要這份文件。"
```

若未提供 prompt argument，`ask` 會從 standard input 讀取內容。

## 附上圖片或文件

`ask` 支援把本機檔案當作附件直接上傳給 ChatGPT，不必透過 pipe 把內容塞進 prompt。

### 附上圖片

使用 `--image`（可重複指定）：

```sh
ask "請描述這張圖片。" --image screenshot.png
ask "比較這兩張圖。" --image v1.png --image v2.png
```

### 附上文件

使用 `--file`（可重複指定）附上 PDF、Word、Excel、純文字、Markdown、CSV、JSON、程式碼等文件：

```sh
ask "請摘要這份 PDF。" --file report.pdf
ask "這份 CSV 有幾筆資料？" --file data.csv
ask "幫我檢查這段程式碼。" --file src/main.rs
```

也可以同時附上圖片與文件：

```sh
ask "對照這張設計圖與規格文件，指出不一致處。" --image design.png --file spec.docx
```

## 切換模型

使用 `--model` 在送出 prompt 前自動切換 ChatGPT 模型或思考強度（不分大小寫與標點）：

```sh
ask "用幾句話介紹 Rust。" --model GPT-5.4
ask "證明這個數學問題。" --model o3
ask "快速翻譯這段話。" --model 即時
```

可用名稱（視帳號權限）：

- **模型**：`GPT-5.5`、`GPT-5.4`、`GPT-5.3`、`o3`
- **思考強度**：`智慧`、`即時`、`中等`、`高`、`超高`、`專業`

## MCP 行為

啟動時，`ask` 會把 MCP 設定寫入：

```text
~/.config/ask-chatgpt/mcp_servers.json
```

預設 server 是 Chrome DevTools MCP：

```json
{
  "mcpServers": {
    "chrome-devtools": {
      "command": "npx",
      "args": [
        "-y",
        "chrome-devtools-mcp@latest",
        "--browser-url=http://127.0.0.1:9223"
      ]
    }
  }
}
```

Rust binary 會透過內建的 `doggy8088/mcp-cli` library dependency 呼叫 Chrome DevTools MCP，不會 shell out 到系統上的 `mcp-cli` 命令。

## 疑難排解

若 Chrome 無法啟動，請確認 Chrome 是否存在於：

```text
/Applications/Google Chrome.app/Contents/MacOS/Google Chrome
```

若 Chrome 存在但 `ask` 無法連線，請關閉此工具先前建立的 Chrome instance 後重試：

```sh
ask login
```

若缺少 `npx`，請先安裝 Node.js，再重新執行：

```sh
make install
```

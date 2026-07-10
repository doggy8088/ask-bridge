# Lessons Learned

## 2026-07-10 Windows quiet MCP 與 JSON fence 碰撞

- Mistake class: incorrect assumption about repository/runtime behavior、missing verification。
- Failure mode: 為隱藏 MCP stderr 而把 Windows stdio server 包進 `cmd.exe /c ... 2>nul`，同時失去可靠的 transport 與失敗診斷；另以第一個三反引號尋找 evaluate_script JSON fence 結尾，遇到回答內的 Markdown code fence 便截斷合法 JSON 字串。
- Detection signal: 同一環境下 quiet 在 `initialize` 回報 server EOF 且沒有 stderr，verbose 直接 `npx.cmd` 則成功；回覆在第一個 ` ```rust ` 前報 `EOF while parsing a string`，瀏覽器已有完整回答但終端只剩 Thread Link。
- Prevention rule: verbosity 不得改變 MCP transport，只能控制 flags/env/呈現；quiet 應在 forwarding 呈現層抑制 stderr，不能在 child transport 前丟棄診斷。結構化資料必須交給 JSON parser 判定值邊界，再獨立驗證 wrapper fence，不得用第一個 Markdown delimiter 截斷內嵌 JSON。
- Tripwires:
  - 單元測試固定斷言 Windows／Unix quiet 與 verbose 使用相同 direct executable transport、不含 shell redirection，且僅 quiet 保留降噪 flags/env。
  - 單元測試固定涵蓋 evaluate_script JSON 字串內含多行 Markdown、程式語言 code fence、雙引號、缺少 closing fence、尾端污染與 malformed response shape。
  - Parser／MCP shape 錯誤不得包含原始 payload；Windows release 前以非 verbose 與 verbose 各執行一次含程式碼區塊的 query，並確認 quiet 不漏 banner、失敗仍有 child stderr 診斷。

## 2026-07-10 Windows Chrome ownership 與登入判斷回歸

- Mistake class: incorrect assumption about repository/runtime behavior、missing verification。
- Failure mode: 把 `Command::spawn()` 回傳的 Chrome launcher PID 當成最終 9223 listener，並假設 WMI/CIM parent-chain 永遠可查；同時以單次 ChatGPT DOM snapshot 決定登入狀態。
- Detection signal: verbose log 同時出現不同的 `recorded PID`／`listener PID`、空的 `ask-bridge owner PIDs`，登入完成後為 `Unknown`，下一次 query 又立即成為 `LoggedOut`。
- Prevention rule: Windows Chrome 啟動完成後必須驗證 listener 來源、記錄實際 listener 與 CDP browser identity；reuse／close 共用同一 ownership snapshot，強殺前重新驗證。登入 UI 必須經 bounded stabilization，未穩定只能回 `Unknown`，不得硬判登出。
- Tripwires:
  - 單元測試固定涵蓋 launcher PID 與 listener PID 不同、WMI 空白列、9223／92230 精確解析、stale identity、mixed listeners 與強殺 identity 改變。
  - 單元測試固定涵蓋 ChatGPT auth path precedence、composer-only provider 差異、未穩定訊號不得成為 `LoggedIn`／`LoggedOut`。
  - Windows release 前執行 login → 保持 Chrome 開啟 → query → graceful close → restart query 的真機流程；若無法執行，必須明確記錄限制，不得只以跨平台單元測試宣稱 session 問題已解決。

翻譯是一個按功能設定的 LLM 任務，只需在伺服器上設定一次。任何用戶端送來的翻譯請求——聊天訊息、提交草稿、編輯器與差異選取範圍——都走同一份設定。

## 啟用功能

開啟 Web 介面的 `LLM 設定` 頁，找到 `翻譯` 卡片：

1. 打開 `啟用 LLM 翻譯` 開關；
2. 選擇 `提供者`——下拉清單只顯示 `附加元件` 頁中**已啟用**的 LLM 附加元件（內建的 **OpenAI Compatible LLM** 提供者可涵蓋大多數情境）；
3. 填寫 `提供者設定` 中的欄位（內建提供者為：`Base URL`、`API Key`、`Model`，以及可選的 `Extra Body`、token 上限、`Timeout`）；
4. 選擇所有譯文使用的 `目標語言`。

功能停用時設定也會儲存，但只有開關打開後伺服器才會套用它。未啟用時，用戶端請求會失敗並提示 *"translation addon is not configured or disabled"*。

![LLM 設定頁中的翻譯卡片——啟用開關、提供者選擇和目標語言](images/translation-server-card.jpg)

## 目標語言

與終端機和 Git 功能不同，翻譯沒有 `自動` 選項——它始終翻譯為清單中某個明確的語言（English、简体中文、繁體中文、Deutsch、Español、Français、日本語、한국어、Nederlands、Polski、Português、Русский、ไทย、Tiếng Việt、العربية、Bahasa Indonesia、Bahasa Melayu、Українська）。此設定對整台伺服器生效；切換後只影響新請求。未設定時預設英語。

![目標語言下拉清單](images/translation-server-language.jpg)

## 重用提供者憑證

`提供者設定` 標題旁的複製/貼上按鈕可以在使用同一提供者附加元件的卡片之間複製欄位值——先在一張卡片（比如 Git 提交訊息）裡選好並填完提供者，點擊**複製**，再在翻譯卡片裡**貼上**。

## 選擇模型

翻譯是短小、對延遲敏感的提示詞——快模型勝過大型推理模型。OpenAI 相容端點（Ollama、vLLM、llama.cpp）後的 **Qwen3.5 9B**，或走 API 的 **DeepSeek V4 Flash** 都很合適。伺服器單次請求最多等待 60 秒；本地慢模型可在提供者欄位中調大 `Timeout`。

> 思考模式會增加數秒延遲，在這裡毫無收益——建議在 `Extra Body` 中關閉，例如 Qwen 用 `{"enable_thinking": false}`，DeepSeek 用 `{"thinking": {"type": "disabled"}}`。

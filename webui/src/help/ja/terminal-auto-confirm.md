サーバー側のターミナルアナライザーはターミナル出力を監視します。コマンドプロンプトを検出し、タスク完了を認識し、LLM でターミナルの表示内容を分類します。`Auto Confirm` は最も使われる機能で、ターミナルが通常の確認プロンプト(`Proceed? [Y/n]` など)で停止したときに Enter を自動送信するため、長いタスクが質問で止まることがなくなります。

## 設定

Web UI の `Terminal Analysis` セクション:

- `Terminal observer` — サーバー側分析のマスタースイッチ;
- `Auto confirm` — **通常の確認** と分類されたプロンプトにのみ Enter を送信します。その他の質問に盲目的に回答することはありません;
- `Analysis delay (seconds)` — エージェントがアイドルになってから分析までの待機時間。タスク間の冗長な LLM 呼び出しを回避します;
- `Confirmation grace (seconds)` — 確認を検出した後にユーザーが介入できる猶予時間;
- `Notify on completion` — 出力がタスク完了と分類されたときにイベントを発行します(通知に接続できます)。

> ページの注意書き:エージェント自身の自動モードが有効な場合は、2 つの自動化が重複して回答しないよう、クライアント側の `Auto confirm` をオフにすることを推奨します。

## LLM 依存

分類は LLM が行います。`LLM` セクションで `Provider` を選び認証情報を入力します(ターミナル分類、Git コミットメッセージ、チャットアシスタントなど AI 機能で共有)。LLM がなくてもアナライザーは精度低下付きで動作しますが、その場合は auto confirm をオフにしてください。

![Terminal Analysis 設定ページ](images/terminal-auto-confirm.jpg)

![LLM 設定ページ — プロバイダーを選び認証情報を入力](images/terminal-auto-confirm-llm.jpg)

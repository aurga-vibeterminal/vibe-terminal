サーバーは Model Context Protocol のホストを内蔵しており、Web UI の `MCP` セクションで設定します。Code Agent（Claude Code、Codex、Kimi Code、Pi、OpenCode、Qwen Code、DeepSeek Harness など）が自身のチャットから接続し、ターミナルの読み取りと操作、さらには EasyEDA Pro の操作まで直接行えます。

## サーバー設定

MCP サーバーを有効にし、以下を設定します：

- `Port`（デフォルト `6180`）と `Bind Address` —— `127.0.0.1` はローカルホスト専用、`0.0.0.0` はネットワークに開放；
- `Access Token` —— localhost 以外からのリクエストに必須の Bearer トークン。localhost からのリクエストは常に許可されます。トークンが空になることはありません —— 新規インストールではランダムなトークンが付属し、サイコロボタンで再生成、コピーボタンで取り出せます；
- `Stream Throttle` —— 接続中のエージェントへの画面ストリーム送信の最小間隔。

変更は `Save` で反映され、サーバーは再起動なしでホットリロードします。

## エンドポイント

各機能セットはそれぞれ独自の URL `http://localhost:{port}/{id}/mcp` で提供されます：

- **Terminal MCP**（`/terminal/mcp`）—— 内蔵のターミナル機能。エージェントはターミナルの一覧表示、可視画面とスクロールバックの読み取り、生入力と名前付きキーの送信、サイズ変更、プロンプト待機が可能です。ツール呼び出しは Terminal ID でターミナルを指定します —— ターミナルを右クリック → `MCP` → `Copy Terminal ID` で ID をコピーし、エージェントのコンテキストに貼り付けてください（詳細は Terminals セクションの Terminal MCP）；
- **EasyEDA Pro MCP**（`/easyeda/mcp`）—— EasyEDA Pro をエージェントに橋渡しする内蔵 MCP アドオン。少数の固定ツールと検索可能な `eda.*` API カタログを備え、WebSocket 経由で EasyEDA コネクター拡張に転送します。まず `Addons` セクションでアドオンを有効にしてください —— インストールするとエンドポイントが一覧に現れ、バッジはアドオンデーモンの接続状態を示します。アドオンの `Timeout` フィールドは各ブリッジ要求の上限です（低速な EDA 操作では大きくしてください）；
- **その他の MCP アドオン** —— インストールされた MCP タイプのアドオンは同じ形式でエンドポイントを追加し、接続/切断バッジがリアルタイム表示されます。

一覧の上のドロップダウンでエージェント CLI を選ぶと、エンドポイント横のコピーボタンがそのアプリ向けの実行可能なセットアップコマンドを渡します。

## エージェントの接続

トークンを手で貼り付ける必要はありません。サーバーは自動生成された `Vibe Terminal MCP Bearer Token` 環境プロファイル（Environment Variables セクションを参照）を維持し、`VTERM_MCP_TOKEN` をエクスポートします。エージェント接続前にターミナルへ適用してから、コピーしたコマンドを使います：

- **シェルコマンド型アプリ**（Claude Code、Codex）—— 通常のターミナルでコピーした `mcp add` コマンドを実行します。Codex は `--bearer-token-env-var` で `VTERM_MCP_TOKEN` からトークンを読むため、コマンドラインに秘密が現れません；
- **会話型アプリ**（Kimi Code、Pi、OpenCode、Qwen Code、DeepSeek Harness）—— プロファイルを適用してアプリを起動し、コピーしたプロンプトを会話に送信します。Agent が MCP 設定を直接更新します。Kimi は `bearerTokenEnvVar` を使い、Pi、OpenCode、Qwen Code は `VTERM_MCP_TOKEN` の Bearer トークンを使うため、トークン自体が会話に表示されることはありません。

Pi と Qwen Code では `/mcp` が接続を確認します。

設定の下の `Command Templates` カードにアプリごとのセットアップコマンドがあります —— エントリの編集、内蔵の上書き、新しいエージェント CLI 用テンプレートの追加が可能で、プレビューはコピー結果そのものです。

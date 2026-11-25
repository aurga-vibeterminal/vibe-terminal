すべては Web UI の `Remote Desktop` セクションにあります。ページにはサーバーのプラットフォームに適用される設定が表示されます。

## メインスイッチ

`Enable Remote Desktop` — オンにすると、接続したリモートクライアントがこのマシンのデスクトップを閲覧し入力を送れます。

## プラットフォーム設定

**Windows**

- `Windows Service` — 画面サーバーを自動開始のシステムサービスとして登録し、ログインセッションなしでも再起動を生き延びます。`Install` / `Uninstall` はサーバー上で UAC プロンプトを起こします(待機中、ボタンに進行状況が表示されます)。ステータスバッジが実行中 / インストール済み / 未インストールを区別します。
- `Headless machines (no monitor)` — 接続された(または起きている)ディスプレイがないと、Windows はキャプチャ可能なデスクトップを保持しません。ページはオープンソースの Virtual Display Driver のダウンロード(ARM64 / x64 自動選択)へのリンクを出します。インストールすれば仮想デスクトップが常に存在します。

![Web UI — Windows の Remote Desktop セクション(サービスとヘッドレスカード)](images/remote-desktop-server-windows.jpg)

**Linux**

- `Unlock on Connect` — ストリーム開始時にロックされたセッションを自動的にロック解除します(環境が対応する場合);
- 画面キャプチャは Portal 経由。`Screen Recording Permission` カードの `Reset Permission` は保存された Portal の同意をクリアし、別の画面を選べるようにします;
- `Auto-start on Login`(`サーバー` セクション内)— ディスプレイマネージャーがあるホストではサーバーはグラフィカルセッションとともに起動し、キャプチャは完全なデスクトップ環境を引き継ぎます。ヘッドレスホストでは、ログイン前のブート時に起動します。ローカルで誰もログインしていない状態でデスクトップにアクセスするには、ディスプレイマネージャー(GDM、SDDM、…)で自動ログインも有効にしてください。セキュリティのため自動画面ロックは有効のままにできます — ロック中もセッションとサーバーは動作し続け、`Unlock on Connect`(または `loginctl unlock-session`)でリモートからセッションを開きます。
![Web UI — 画面収録許可カード(Linux)](images/remote-desktop-server-permission.jpg)


**macOS**

- `Screen Recording Permission` — システムはキャプチャに明示的な承認を要求します。`Grant Permission` でシステムダイアログを開きます。`Reset Permission` は 固まった状態から復旧します。

## 画面ストリーミングツール(Android サポート)

`Android SDK directory` と `Android Studio directory` が adb とエミュレーターを特定します。自動検出のままにする場合は空にします。設定すると、ページに解決されたパスとその由来(auto / env / override)が表示されます。`Reset to Auto-Detect` でデフォルトに戻せます。これらで何が可能になるかは Android デバイスを参照してください。

## ストリームエンコーディング

- `Encoder` — Auto または特定のバックエンド。サーバーは使用可能なエンコーダーを一覧表示し、利用不可のものは理由付きでグレーアウトします;
- `Frame rate` — 60 / 30 / 15 fps;
- `Max resolution` — キャプチャ側の上限(4K / 2.5K / 2K);
- `Bitrate` — `Auto` は解像度に応じて面積スケール(1080p までベースレート、4K で約 4 倍、40 Mbps 上限)。固定値は実行中ストリームに即座に適用され、エンコーダー / fps / 解像度は以降に開始されるストリームに適用されます;
- カードのフッターに現在有効なエンコーダー / fps / 解像度 / ビットレートが表示されます。

![Web UI — ストリームエンコーディング設定](images/remote-desktop-server-stream.jpg)

`Save` をクリックして変更を適用します。

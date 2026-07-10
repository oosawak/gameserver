# 🎮 MOBA ゲーム - シンプル対戦ゲーム

複数プレイヤーでリアルタイムに対戦できるシンプルなMOBAゲームです。

## 📋 仕様

- **プレイ方法**: WASD キーで移動、SPACE キーでシュート
- **マップ**: 1000 × 1000 のアリーナ
- **ゲーム性**: 複数プレイヤーが移動・シューティング
- **HP**: 初期値 100、弾が当たると -10
- **リアルタイム同期**: WebSocket 経由で 60 Hz で同期

## 🚀 実行方法

### 1. サーバーの起動

```bash
cd game/MOBA
cargo run --release --bin moba-server
```

出力例：
```
🎮 MOBA Server started on 127.0.0.1:8080
```

### 2. ブラウザでゲーム画面を開く

サーバー起動後、ブラウザで以下を開きます：

```
file:///home/oosawak/Workspace/gameserver/game/MOBA/www/index.html
```

または、簡易HTTPサーバーで開く：

```bash
cd game/MOBA/www
python3 -m http.server 8000
```

その後 `http://localhost:8000` をブラウザで開く

### 3. ゲーム参加

1. プレイヤー名を入力（デフォルト: Player1）
2. 「参加」ボタンをクリック
3. ゲームが開始

## ⌨️ 操作方法

| キー | 動作 |
|------|------|
| W | 上に移動 |
| A | 左に移動 |
| S | 下に移動 |
| D | 右に移動 |
| SPACE | シュート（弾発射） |

## 🎯 ゲームルール

- **プレイヤー（緑）**: あなたのキャラ
- **プレイヤー（青）**: 他のプレイヤー
- **弾**: SPACE で発射。他のプレイヤーに当たると -10 ダメージ
- **HP 0**: ゲームオーバー（再参加で復活）

## 🏗️ ファイル構成

```
game/MOBA/
├── Cargo.toml              # Rust プロジェクト設定
├── README.md               # このファイル
├── src/
│   ├── main.rs            # WebSocket サーバー
│   └── moba.rs            # MOBA ゲームモード
└── www/
    ├── index.html         # ゲーム画面（HTML/CSS）
    └── client.js          # ゲームロジック（JavaScript）
```

## 🔧 コード概要

### サーバー側（Rust）

- **src/main.rs**: WebSocket サーバー。複数クライアント接続を管理
- **src/moba.rs**: GameMode トレイト実装。物理演算・衝突判定

### クライアント側（JavaScript）

- **www/index.html**: Canvas + UI
- **www/client.js**: 
  - キー入力処理
  - Canvas 描画
  - WebSocket 通信

## 📊 通信フォーマット

### クライアント → サーバー

```json
{
  "type": "join",
  "name": "プレイヤー名"
}
```

```json
{
  "type": "input",
  "move_x": -1.0,
  "move_y": 0.0,
  "action1": true
}
```

### サーバー → クライアント

```json
{
  "type": "state",
  "state": {
    "tick": 120,
    "players": [
      {
        "id": 123,
        "x": 500.0,
        "y": 300.0,
        "health": 85.0,
        "max_health": 100.0
      }
    ]
  }
}
```

## 🎨 UI 機能

- **ゲーム情報**: Tick・プレイヤー数・HP・位置情報
- **プレイヤー一覧**: 全プレイヤーの状態表示
- **ヘルスバー**: ビジュアルなHP表示

## 🐛 トラブルシューティング

### サーバーが起動しない

```bash
# 依存関係確認
cargo check --manifest-path game/MOBA/Cargo.toml

# ビルド実行
cargo build --manifest-path game/MOBA/Cargo.toml
```

### ブラウザで接続できない

- WebSocket が `ws://localhost:8080` を指しているか確認
- ファイアウォール設定を確認
- ブラウザのコンソールでエラーを確認（F12）

### 複数プレイヤーで同期しない

サーバーが複数接続を受け入れているか確認：

```bash
RUST_LOG=debug cargo run --bin moba-server
```

## 📈 今後の拡張案

- [ ] チーム分け（Red vs Blue）
- [ ] スコアボード
- [ ] ゲーム終了判定
- [ ] パワーアップアイテム
- [ ] 異なるキャラクタースキル
- [ ] サウンドエフェクト
- [ ] マルチマップ
- [ ] ランク・レーティング

## 📝 ライセンス

MIT

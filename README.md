
# gameserver

A lightweight and extensible multiplayer game server.  
This project aims to provide a simple foundation for real-time communication, room management, and session handling.

## Features
- Multiplayer game server foundation
- Real-time communication (e.g. WebSocket / TCP)
- Room lifecycle management (create/join/leave)
- Scalable architecture for future growth

## Keywords
gameserver, game server, multiplayer, realtime, websocket, matchmaking



# Game Server - 共通ゲームエンジン

Rust + WASM + WebRTC を使用した、複数ジャンルのリアルタイム対戦ゲームを同一基盤で構築するゲームエンジン。

## 🎮 対応ゲームモード

- **MOBAモード** - ブロスタ風アクション
- **サッカー** - ボールキック・ゴール判定
- **バスケットボール** - シュート・バスケット判定
- その他のスポーツ・アクションゲーム（拡張可能）

## 📁 プロジェクト構成

```
gameserver/
├── DESIGN.md              # 設計資料（詳細）
├── Cargo.toml             # ワークスペース定義
├── core/                  # Core Layer（完全共通）
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── entity.rs      # Entity, EntityKind, Transform, Health
│       ├── world.rs       # World（ゲーム世界）
│       ├── physics.rs     # 物理演算
│       ├── map.rs         # マップ・障害物管理
│       ├── gamemode.rs    # GameMode トレイト
│       └── input.rs       # 入力定義
├── server/                # Server Layer（Rust）
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs        # サーバーメインループ
│       └── modes/
│           ├── mod.rs
│           ├── moba.rs    # MOBAモード実装
│           ├── soccer.rs  # サッカーモード実装
│           └── basketball.rs # バスケットボール実装
└── client/                # Client Layer（WASM）
    ├── Cargo.toml
    └── src/
        └── lib.rs         # WASM クライアント（予測＋補正）
```

## 🏗️ アーキテクチャ

### Core Layer（共通）

**完全な型安全性とシリアライゼーションで共有される**

- `Entity`: プレイヤー・ボール・弾など統一管理
- `World`: ゲーム世界・全エンティティ保持
- `Physics`: 共通物理演算（速度・摩擦）
- `Map`: 壁・ゴール・ブッシュなど
- `GameMode`: ゲームモード定義トレイト

### Server Layer（Rust）

- **Authoritative Simulation**: サーバーが唯一の真実
- **GameMode差し替え**: 3つのモード実装済み（MOBA, Soccer, Basketball）
- **ゲームループ**: 60 Hz で物理・入力処理・衝突判定

### Client Layer（WASM）

- **ローカル予測**: 入力に基づいた即座の応答
- **補正（Reconciliation）**: サーバー状態による修正
- **軽量実装**: Canvas/WebGL用の描画準備

## 🚀 ビルド方法

### サーバーのビルド・実行

```bash
# ビルド
cargo build --release --bin gameserver

# 実行
cargo run --bin gameserver

# ログ出力
RUST_LOG=debug cargo run --bin gameserver
```

### クライアント（WASM）のビルド

```bash
# wasm-pack インストール
cargo install wasm-pack

# ビルド
wasm-pack build client --target web --release

# 生成物: client/pkg/
```

### テスト

```bash
cargo test --all
```

## 📦 依存関係

- **tokio**: 非同期ランタイム
- **serde/serde_json**: シリアライゼーション
- **wasm-bindgen**: Rust ↔ WebAssembly 相互運用
- **web-sys**: Web API バインディング
- **webrtc**: WebRTC 通信（計画中）

## 🎯 主な特徴

### 1. ゲームモード差し替え

`GameMode` トレイトを実装するだけで新モード追加可能：

```rust
pub trait GameMode: Send + Sync {
    fn init(&mut self, world: &mut World);
    fn handle_input(&mut self, world: &mut World, input: PlayerInput);
    fn update(&mut self, world: &mut World, dt: f32);
    fn name(&self) -> &'static str;
}
```

### 2. 統一エンティティ管理

プレイヤー・ボール・弾・障害物をすべて `Entity` で管理。

### 3. クライアント予測＋補正

- 入力で即座にローカルエンティティ更新
- サーバーのスナップショット受信で補正
- ネットワーク遅延の隠蔽

## 🔧 設定・カスタマイズ

### ゲームモード変更

`server/src/main.rs` の `main()` 関数で：

```rust
let mut game_mode: Box<dyn GameMode> = Box::new(SoccerMode::new()); // サッカーに変更
```

### Tick Rate 変更

```rust
const TICK_RATE: f64 = 60.0; // Hz単位
const DT: f32 = (1.0 / TICK_RATE) as f32;
```

### マップ・スポーン位置変更

`modes/*/rs` 内の `init()` 関数を編集。

## 📖 参考資料

- [設計資料](./DESIGN.md) - 詳細なアーキテクチャと実装方針
- Rust Book: https://doc.rust-lang.org/book/
- WASM Book: https://rustwasm.github.io/docs/book/

## 🚧 今後の拡張

- [ ] WebRTC DataChannel 統合
- [ ] クライアント補正ロジック完成
- [ ] ネットワークプロトコル（バイナリ化）
- [ ] スコア・ゲーム終了判定
- [ ] マップエディタ
- [ ] キャラクターカスタマイズ
- [ ] チーム・ランク機能

## 📝 ライセンス

MIT

## 👤 作者

oosawak

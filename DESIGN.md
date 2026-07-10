# 共通ゲームエンジン設計資料（Rust + WASM + WebRTC）

## 1. 目的

本エンジンは以下の複数ジャンルのリアルタイム対戦ゲームを同一の基盤で構築できることを目的とする。

- MOBAモード（ブロスタ風アクション）
- サッカー
- バスケットボール
- その他スポーツ・アクションゲーム

共通化の中心は以下の3点：

- リアルタイム同期（WebRTC）
- 共通物理・エンティティ管理（Rust）
- クライアント予測＋補正（WASM）

## 2. 全体アーキテクチャ

```
┌──────────────────────────────┐
│          Client (WASM)        │
│  - 入力取得                   │
│  - 予測(Prediction)           │
│  - 補正(Reconciliation)       │
│  - 描画(Canvas/WebGL)         │
└───────────────▲──────────────┘
                │ WebRTC DataChannel
┌───────────────┴──────────────┐
│         Net Layer (共通)       │
│  - 入力送信                    │
│  - 状態差分受信                │
│  - バイナリプロトコル          │
└───────────────▲──────────────┘
                │
┌───────────────┴──────────────┐
│        Server (Rust)           │
│  - authoritative simulation     │
│  - 物理・当たり判定            │
│  - ゲームループ                │
│  - GameMode 差し替え           │
└───────────────▲──────────────┘
                │
┌───────────────┴──────────────┐
│        Core Layer (共通)       │
│  - World                       │
│  - Entity                      │
│  - Physics                     │
│  - Map                         │
└──────────────────────────────┘
```

## 3. Core Layer（完全共通）

### 3.1 World

ゲーム世界の全データを保持する。

```rust
struct World {
    entities: HashMap<EntityId, Entity>,
    map: Map,
    tick: u64,
}
```

### 3.2 Entity

プレイヤー・ボール・弾・障害物などすべてを統一管理。

```rust
enum EntityKind {
    Player,
    Ball,
    Projectile,
    Obstacle,
}

struct Entity {
    id: EntityId,
    kind: EntityKind,
    transform: Transform,
    physics: Physics,
    health: Option<Health>,
    owner: Option<EntityId>, // ボール所持など
}
```

### 3.3 Physics

共通物理。

```rust
struct Physics {
    vx: f32,
    vy: f32,
    mass: f32,
    friction: f32,
}
```

### 3.4 Map

壁・ゴール・ブッシュなどを共通構造で持つ。

## 4. GameMode Layer（差し替え可能）

### 4.1 インターフェース

```rust
trait GameMode {
    fn init(&mut self, world: &mut World);
    fn handle_input(&mut self, world: &mut World, input: PlayerInput);
    fn update(&mut self, world: &mut World, dt: f32);
}
```

### 4.2 MOBAモード（旧ブロスタ風）

```rust
struct MobaMode;

impl GameMode for MobaMode {
    fn init(&mut self, world: &mut World) {
        // スポーン配置・壁生成
    }

    fn handle_input(&mut self, world: &mut World, input: PlayerInput) {
        apply_movement(world, input.player_id, input.move_x, input.move_y);
        if input.action1 {
            spawn_projectile(world, input.player_id);
        }
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        update_physics(world, dt);
        resolve_projectile_hits(world);
    }
}
```

### 4.3 サッカーモード

```rust
struct SoccerMode {
    ball_id: EntityId,
}

impl GameMode for SoccerMode {
    fn init(&mut self, world: &mut World) {
        self.ball_id = spawn_ball(world);
    }

    fn handle_input(&mut self, world: &mut World, input: PlayerInput) {
        apply_movement(world, input.player_id, input.move_x, input.move_y);
        if input.action1 {
            try_kick_ball(world, input.player_id, self.ball_id);
        }
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        update_physics(world, dt);
        update_ball_possession(world, self.ball_id);
        if check_goal(world, self.ball_id) {
            reset_positions(world, self.ball_id);
        }
    }
}
```

### 4.4 バスケモード

```rust
struct BasketballMode {
    ball_id: EntityId,
}

impl GameMode for BasketballMode {
    fn init(&mut self, world: &mut World) {
        self.ball_id = spawn_ball(world);
    }

    fn handle_input(&mut self, world: &mut World, input: PlayerInput) {
        apply_movement(world, input.player_id, input.move_x, input.move_y);
        if input.action1 {
            try_shoot_ball(world, input.player_id, self.ball_id);
        }
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        update_physics(world, dt);
        if check_shot_success(world, self.ball_id) {
            reset_positions(world, self.ball_id);
        }
    }
}
```

## 5. Net Layer（WebRTC / WebSocket 共通）

### 5.1 入力送信（軽量）

```rust
struct PlayerInput {
    player_id: EntityId,
    move_x: f32,
    move_y: f32,
    action1: bool,
    action2: bool,
}
```

### 5.2 状態差分送信（軽量）

- プレイヤー位置
- ボール位置
- 弾位置
- HP
- tick

### 5.3 サーバー authoritative

```rust
loop {
    let inputs = collect_inputs();
    for input in inputs {
        mode.handle_input(&mut world, input);
    }

    mode.update(&mut world, dt);

    let diff = make_state_diff(&world);
    broadcast(diff);
}
```

## 6. Client Layer（WASM）

### 6.1 予測＋補正

```rust
fn update(dt: f32) {
    let input = read_input();
    send_input(input);

    apply_local_prediction(input, dt);

    if let Some(snapshot) = take_snapshot() {
        reconcile(snapshot);
    }

    render();
}
```

## 7. 拡張性

- 新モード追加 → `GameMode` を実装するだけ
- 新キャラ追加 → `CharacterStats` を増やすだけ
- 新スポーツ追加 → ボールロジックを差し替えるだけ
- 新マップ追加 → `Map` を差し替えるだけ

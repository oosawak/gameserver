const canvas = document.getElementById('gameCanvas');
const ctx = canvas.getContext('2d');

const MAP_WIDTH = 1000;
const MAP_HEIGHT = 1000;
const SCALE = Math.min(canvas.width / MAP_WIDTH, canvas.height / MAP_HEIGHT);
const WORLD_OFFSET_X = (canvas.width - MAP_WIDTH * SCALE) / 2;
const WORLD_OFFSET_Y = (canvas.height - MAP_HEIGHT * SCALE) / 2;
const SETTINGS_STORAGE_KEY = 'mobaClientSettings';
const DEFAULT_SETTINGS = {
    inputIntervalMs: 50,
    uiIntervalMs: 150,
    speed: 400,
    gridCache: true
};

const gridCanvas = document.createElement('canvas');
gridCanvas.width = canvas.width;
gridCanvas.height = canvas.height;
const gridCtx = gridCanvas.getContext('2d');

let ws = null;
let gameState = null;
let clientId = null;
let entityId = null;
let playerName = '';
let isConnected = false;
let lastInputSendAt = 0;
let lastSentInput = { move_x: null, move_y: null, action1: null };
let lastUiUpdateAt = 0;
let lastSpeedSendAt = 0;
const autoMoveSelections = new Set();
const playerFacingAngles = new Map();
let settings = loadSettings();

const keys = {
    w: false,
    a: false,
    s: false,
    d: false,
    up: false,
    down: false,
    left: false,
    right: false,
    space: false
};

// イベントリスナー
document.getElementById('joinBtn').addEventListener('click', joinGame);
document.getElementById('leaveBtn').addEventListener('click', leaveGame);
document.getElementById('clearBtn').addEventListener('click', clearParticipants);
document.getElementById('inputIntervalRange').addEventListener('input', syncSettingsFromUi);
document.getElementById('uiIntervalRange').addEventListener('input', syncSettingsFromUi);
document.getElementById('speedRange').addEventListener('input', syncSettingsFromUi);
document.getElementById('toggleGridCache').addEventListener('change', syncSettingsFromUi);

applySettingsToUi();

function loadSettings() {
    try {
        const raw = localStorage.getItem(SETTINGS_STORAGE_KEY);
        if (!raw) return { ...DEFAULT_SETTINGS };

        const parsed = JSON.parse(raw);
        return {
            ...DEFAULT_SETTINGS,
            ...parsed
        };
    } catch (e) {
        return { ...DEFAULT_SETTINGS };
    }
}

function saveSettings() {
    localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings));
}

function applySettingsToUi() {
    document.getElementById('inputIntervalRange').value = settings.inputIntervalMs;
    document.getElementById('uiIntervalRange').value = settings.uiIntervalMs;
    document.getElementById('speedRange').value = settings.speed;
    document.getElementById('toggleGridCache').checked = settings.gridCache;
    document.getElementById('inputIntervalValue').textContent = `${settings.inputIntervalMs}ms`;
    document.getElementById('uiIntervalValue').textContent = `${settings.uiIntervalMs}ms`;
    document.getElementById('speedValue').textContent = settings.speed;
}

function syncSettingsFromUi() {
    const inputIntervalMs = Number(document.getElementById('inputIntervalRange').value);
    const uiIntervalMs = Number(document.getElementById('uiIntervalRange').value);
    const speed = Number(document.getElementById('speedRange').value);
    settings = {
        inputIntervalMs: Number.isFinite(inputIntervalMs) ? inputIntervalMs : DEFAULT_SETTINGS.inputIntervalMs,
        uiIntervalMs: Number.isFinite(uiIntervalMs) ? uiIntervalMs : DEFAULT_SETTINGS.uiIntervalMs,
        speed: Number.isFinite(speed) ? speed : DEFAULT_SETTINGS.speed,
        gridCache: document.getElementById('toggleGridCache').checked
    };
    document.getElementById('inputIntervalValue').textContent = `${settings.inputIntervalMs}ms`;
    document.getElementById('uiIntervalValue').textContent = `${settings.uiIntervalMs}ms`;
    document.getElementById('speedValue').textContent = settings.speed;
    saveSettings();
    lastInputSendAt = 0;
    lastSentInput = { move_x: null, move_y: null, action1: null };
    lastUiUpdateAt = 0;
    sendSpeedToServer();
}

function sendSpeedToServer() {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;

    const now = performance.now();
    if (now - lastSpeedSendAt < 100) return;

    ws.send(JSON.stringify({
        type: 'set_speed',
        speed: settings.speed
    }));
    lastSpeedSendAt = now;
}

function sendAutoMoveToServer(entityId, enabled) {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;

    ws.send(JSON.stringify({
        type: 'set_auto_move',
        entity_id: entityId,
        enabled
    }));
}

document.addEventListener('keydown', (e) => {
    const key = e.key.toLowerCase();
    if (key === 'w') keys.w = true;
    if (key === 'a') keys.a = true;
    if (key === 's') keys.s = true;
    if (key === 'd') keys.d = true;
    if (key === 'arrowup') {
        keys.up = true;
        e.preventDefault();
    }
    if (key === 'arrowdown') {
        keys.down = true;
        e.preventDefault();
    }
    if (key === 'arrowleft') {
        keys.left = true;
        e.preventDefault();
    }
    if (key === 'arrowright') {
        keys.right = true;
        e.preventDefault();
    }
    if (key === ' ') {
        keys.space = true;
        e.preventDefault();
    }
});

document.addEventListener('keyup', (e) => {
    const key = e.key.toLowerCase();
    if (key === 'w') keys.w = false;
    if (key === 'a') keys.a = false;
    if (key === 's') keys.s = false;
    if (key === 'd') keys.d = false;
    if (key === 'arrowup') keys.up = false;
    if (key === 'arrowdown') keys.down = false;
    if (key === 'arrowleft') keys.left = false;
    if (key === 'arrowright') keys.right = false;
    if (key === ' ') keys.space = false;
});

function joinGame() {
    playerName = document.getElementById('playerName').value.trim() || 'Player';
    if (!playerName) playerName = 'Player';

    const wsHost = location.hostname || 'localhost';
    const wsScheme = location.protocol === 'https:' ? 'wss:' : 'ws:';
    ws = new WebSocket(`${wsScheme}//${wsHost}:8888`);

    ws.onopen = () => {
        console.log('Connected to server');
        isConnected = true;
        updateStatus();
        sendSpeedToServer();

        ws.send(JSON.stringify({
            type: 'join',
            name: playerName
        }));

        document.getElementById('playerName').disabled = true;
        document.getElementById('joinBtn').disabled = true;
        document.getElementById('leaveBtn').disabled = false;
        document.getElementById('clearBtn').disabled = false;

        gameLoop();
    };

    ws.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data);

            if (data.type === 'joined') {
                clientId = data.client_id;
                entityId = data.entity_id;
                console.log(`Joined as client ${clientId}, entity ${entityId}`);
                gameState = data.state;
            } else if (data.type === 'state') {
                gameState = data.state;
                updateUI();
            }
        } catch (e) {
            console.error('Failed to parse message:', e);
        }
    };

    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        isConnected = false;
        updateStatus();
    };

    ws.onclose = () => {
        console.log('Disconnected from server');
        isConnected = false;
        updateStatus();
        document.getElementById('playerName').disabled = false;
        document.getElementById('joinBtn').disabled = false;
        document.getElementById('leaveBtn').disabled = true;
        document.getElementById('clearBtn').disabled = true;
        clientId = null;
        entityId = null;
        autoMoveSelections.clear();
    };
}

function leaveGame() {
    if (ws) {
        ws.send(JSON.stringify({
            type: 'leave'
        }));
        ws.close();
    }
}

function clearParticipants() {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    if (!confirm('参加者を全員クリアします。よろしいですか?')) return;

    ws.send(JSON.stringify({
        type: 'clear_participants'
    }));

    autoMoveSelections.clear();
    playerFacingAngles.clear();
    gameState = {
        tick: gameState ? gameState.tick : 0,
        players: [],
        entities: []
    };
    updateUI();
}

function drawPlayerTriangle(x, y, angle, color) {
    const size = 22 * SCALE;
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(angle);
    ctx.shadowColor = color;
    ctx.shadowBlur = 10;
    ctx.fillStyle = color;
    ctx.strokeStyle = '#06110a';
    ctx.lineWidth = Math.max(1, 1.5 * SCALE);
    ctx.beginPath();
    ctx.moveTo(size, 0);
    ctx.lineTo(-size * 0.8, -size * 0.7);
    ctx.lineTo(-size * 0.8, size * 0.7);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
    ctx.restore();
}

function gameLoop() {
    sendInput();
    render();
    updateUI();

    if (isConnected) {
        requestAnimationFrame(gameLoop);
    }
}

function sendInput() {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;

    let moveX = 0;
    let moveY = 0;

    if (keys.w || keys.up) moveY -= 1;
    if (keys.s || keys.down) moveY += 1;
    if (keys.a || keys.left) moveX -= 1;
    if (keys.d || keys.right) moveX += 1;

    if (moveX !== 0 || moveY !== 0) {
        const len = Math.sqrt(moveX * moveX + moveY * moveY);
        moveX /= len;
        moveY /= len;
    }

    const now = performance.now();
    const input = {
        move_x: moveX,
        move_y: moveY,
        action1: keys.space
    };

    if (settings.inputIntervalMs <= 0) {
        ws.send(JSON.stringify({
            type: 'input',
            ...input
        }));
        lastInputSendAt = now;
        lastSentInput = input;
        return;
    }

    const changed =
        input.move_x !== lastSentInput.move_x ||
        input.move_y !== lastSentInput.move_y ||
        input.action1 !== lastSentInput.action1;
    const shouldKeepAlive = now - lastInputSendAt >= settings.inputIntervalMs;

    if (!changed && !shouldKeepAlive) return;

    ws.send(JSON.stringify({
        type: 'input',
        ...input
    }));
    lastInputSendAt = now;
    lastSentInput = input;
}

function render() {
    // 背景
    ctx.fillStyle = '#000';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    if (!settings.gridCache) {
        gridCtx.clearRect(0, 0, canvas.width, canvas.height);
        gridCtx.strokeStyle = '#222';
        gridCtx.lineWidth = 1;
        for (let x = 0; x <= MAP_WIDTH; x += 100) {
            const sx = WORLD_OFFSET_X + x * SCALE;
            gridCtx.beginPath();
            gridCtx.moveTo(sx, 0);
            gridCtx.lineTo(sx, canvas.height);
            gridCtx.stroke();
        }
        for (let y = 0; y <= MAP_HEIGHT; y += 100) {
            const sy = WORLD_OFFSET_Y + y * SCALE;
            gridCtx.beginPath();
            gridCtx.moveTo(WORLD_OFFSET_X, sy);
            gridCtx.lineTo(WORLD_OFFSET_X + MAP_WIDTH * SCALE, sy);
            gridCtx.stroke();
        }
        ctx.drawImage(gridCanvas, 0, 0);
    } else {
        if (!gridCanvas.dataset.ready) {
            gridCtx.strokeStyle = '#222';
            gridCtx.lineWidth = 1;
            for (let x = 0; x <= MAP_WIDTH; x += 100) {
                const sx = WORLD_OFFSET_X + x * SCALE;
                gridCtx.beginPath();
                gridCtx.moveTo(sx, 0);
                gridCtx.lineTo(sx, canvas.height);
                gridCtx.stroke();
            }
            for (let y = 0; y <= MAP_HEIGHT; y += 100) {
                const sy = WORLD_OFFSET_Y + y * SCALE;
                gridCtx.beginPath();
                gridCtx.moveTo(WORLD_OFFSET_X, sy);
                gridCtx.lineTo(WORLD_OFFSET_X + MAP_WIDTH * SCALE, sy);
                gridCtx.stroke();
            }
            gridCanvas.dataset.ready = '1';
        }
        ctx.drawImage(gridCanvas, 0, 0);
    }

    if (!gameState) return;

    const entities = Array.isArray(gameState.entities) ? gameState.entities : [];
    const renderEntities = entities.length > 0
        ? entities
        : gameState.players.map(player => ({
            id: player.id,
            x: player.x,
            y: player.y,
            kind: 'player',
            vx: player.vx ?? 0,
            vy: player.vy ?? 0,
            health: player.health,
            max_health: player.max_health
        }));

    // プレイヤー・弾の描画
    renderEntities.forEach(entity => {
        const x = WORLD_OFFSET_X + entity.x * SCALE;
        const y = WORLD_OFFSET_Y + entity.y * SCALE;

        if (entity.kind === 'projectile') {
            const size = Math.max(5, 7 * SCALE);
            const vx = entity.vx ?? 0;
            const vy = entity.vy ?? 0;
            const len = Math.sqrt(vx * vx + vy * vy) || 1;
            const trail = 10 * SCALE;
            const tailX = x - (vx / len) * trail;
            const tailY = y - (vy / len) * trail;
            ctx.save();
            ctx.shadowColor = '#ffef8a';
            ctx.shadowBlur = 12;
            ctx.fillStyle = '#ffe14d';
            ctx.strokeStyle = '#3b2a00';
            ctx.lineWidth = Math.max(1, 1.5 * SCALE);
            ctx.beginPath();
            ctx.moveTo(tailX, tailY);
            ctx.lineTo(x, y);
            ctx.stroke();
            ctx.beginPath();
            ctx.arc(x, y, size, 0, Math.PI * 2);
            ctx.fill();
            ctx.stroke();
            ctx.restore();
            return;
        }

        const isMe = entity.id === entityId;
        const angle = (() => {
            const vx = entity.vx ?? 0;
            const vy = entity.vy ?? 0;
            const current = Math.atan2(vy, vx);
            const previous = playerFacingAngles.get(entity.id);
            if (Math.abs(vx) > 0.01 || Math.abs(vy) > 0.01) {
                playerFacingAngles.set(entity.id, current);
                return current;
            }
            return previous ?? 0;
        })();

        drawPlayerTriangle(x, y, angle, isMe ? '#00ff00' : '#0080ff');

        ctx.fillStyle = '#ff0000';
        ctx.fillRect(x - 20 * SCALE, y - 30 * SCALE, 40 * SCALE, 5 * SCALE);

        ctx.fillStyle = '#00ff00';
        const healthRatio = entity.max_health > 0 ? entity.health / entity.max_health : 1;
        ctx.fillRect(x - 20 * SCALE, y - 30 * SCALE, 40 * SCALE * healthRatio, 5 * SCALE);

        ctx.fillStyle = isMe ? '#ffff00' : '#aaaaaa';
        ctx.font = 'bold 12px Arial';
        ctx.textAlign = 'center';
        ctx.fillText(`ID:${entity.id}`, x, y + 35 * SCALE);
    });

    // Tick 表示
    ctx.fillStyle = '#00ff00';
    ctx.font = 'bold 14px Arial';
    ctx.textAlign = 'left';
    ctx.fillText(`Tick: ${gameState.tick}`, 10, 20);

    // 操作説明
    ctx.font = '12px Arial';
    ctx.fillText('WASD: Move | SPACE: Shoot', 10, canvas.height - 10);
}

function updateUI() {
    if (!gameState) return;

    const now = performance.now();
    if (settings.uiIntervalMs > 0) {
        if (now - lastUiUpdateAt < settings.uiIntervalMs) return;
        lastUiUpdateAt = now;
    }

    document.getElementById('tick').textContent = gameState.tick;
    document.getElementById('playerCount').textContent = gameState.players.length;

    // 自分のプレイヤーを探す
    const myPlayer = gameState.players.find(p => p.id === entityId);
    if (myPlayer) {
        const healthPercent = myPlayer.max_health > 0 ? (myPlayer.health / myPlayer.max_health) * 100 : 0;
        document.getElementById('healthFill').style.width = healthPercent + '%';
        document.getElementById('healthText').textContent =
            `${Math.round(myPlayer.health)} / ${Math.round(myPlayer.max_health)}`;
    } else {
        document.getElementById('healthFill').style.width = '0%';
        document.getElementById('healthText').textContent = '0 / 0';
    }

    // プレイヤーリスト
    const playersList = document.getElementById('playersList');
    playersList.innerHTML = '';
    const currentIds = new Set();
    gameState.players.forEach(player => {
        currentIds.add(player.id);
        const isMe = player.id === entityId;
        const item = document.createElement('div');
        item.className = 'player-item';
        item.innerHTML = `
            <div>
                <strong>${isMe ? '► あなた' : '• 他プレイヤー'}</strong>
                <label style="float:right; font-size:0.85em;">
                    <input type="checkbox" data-player-id="${player.id}" ${autoMoveSelections.has(player.id) ? 'checked' : ''}>
                    自動
                </label><br>
                ID: ${player.id}<br>
                位置: (${Math.round(player.x)}, ${Math.round(player.y)})<br>
                HP: ${Math.round(player.health)}/${Math.round(player.max_health)}
            </div>
        `;
        playersList.appendChild(item);

        const checkbox = item.querySelector('input[type="checkbox"]');
        checkbox.addEventListener('change', () => {
            const enabled = checkbox.checked;
            if (enabled) {
                autoMoveSelections.add(player.id);
            } else {
                autoMoveSelections.delete(player.id);
            }
            sendAutoMoveToServer(player.id, enabled);
        });
    });

    for (const id of [...autoMoveSelections]) {
        if (!currentIds.has(id)) {
            autoMoveSelections.delete(id);
        }
    }
}

function updateStatus() {
    const statusEl = document.getElementById('status');
    if (isConnected && clientId !== null) {
        statusEl.className = 'status connected';
        statusEl.textContent = `● オンライン (Client ID: ${clientId})`;
    } else {
        statusEl.className = 'status disconnected';
        statusEl.textContent = '● オフライン';
    }
}

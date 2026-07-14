// platformer.js - Минимальный платформер на JavaScript (Node.js)
const fs = require('fs');
const keypress = require('keypress');
const readline = require('readline');

const WIDTH = 40;
const HEIGHT = 12;
const GRAVITY = 0.3;
const JUMP_SPEED = -5.5;
const PLAYER = '@';
const WALL = '#';
const COIN = '$';
const EXIT = 'X';
const RECORD_FILE = 'platformer_record.json';

const LEVEL_RAW = [
    '                                        ',
    '                                        ',
    '          #   #                         ',
    '   $      #   #     #   #              ',
    '   @      #   #  $  #   #   X          ',
    '########   ########   #############    ',
];
while (LEVEL_RAW.length < HEIGHT) LEVEL_RAW.push(' '.repeat(WIDTH));
const LEVEL = LEVEL_RAW.map(row => row.padEnd(WIDTH, ' ').split(''));

class Platformer {
    constructor() {
        this.width = WIDTH;
        this.height = HEIGHT;
        this.level = LEVEL.map(row => [...row]);
        this.playerX = 0;
        this.playerY = 0;
        this.vx = 0;
        this.vy = 0;
        this.onGround = false;
        this.coins = 0;
        this.totalCoins = 0;
        this.exitX = 0;
        this.exitY = 0;
        this.findPlayerAndExit();
        this.gameOver = false;
        this.won = false;
        this.paused = false;
        this.running = true;
        this.startTime = Date.now();
        this.record = this.loadRecord();
        this.lastUpdate = Date.now();
        this.timer = null;
    }

    findPlayerAndExit() {
        for (let y = 0; y < this.level.length; y++) {
            for (let x = 0; x < this.level[y].length; x++) {
                const ch = this.level[y][x];
                if (ch === PLAYER) {
                    this.playerX = x;
                    this.playerY = y;
                    this.level[y][x] = ' ';
                } else if (ch === EXIT) {
                    this.exitX = x;
                    this.exitY = y;
                } else if (ch === COIN) {
                    this.totalCoins++;
                }
            }
        }
    }

    loadRecord() {
        try {
            const data = fs.readFileSync(RECORD_FILE, 'utf8');
            return JSON.parse(data).record || null;
        } catch { return null; }
    }

    saveRecord(time) {
        fs.writeFileSync(RECORD_FILE, JSON.stringify({ record: time }));
    }

    jump() {
        if (this.onGround && !this.gameOver && !this.paused) {
            this.vy = JUMP_SPEED;
            this.onGround = false;
        }
    }

    collides(x, y) {
        if (x < 0 || x >= this.width || y < 0 || y >= this.height) return true;
        return this.level[y][x] === WALL;
    }

    getLeftCollision(x, y) {
        while (this.collides(x, y) && x >= 0) x--;
        return x + 1;
    }
    getRightCollision(x, y) {
        while (this.collides(x, y) && x < this.width) x++;
        return x - 1;
    }
    getTopCollision(x, y) {
        while (this.collides(x, y) && y >= 0) y--;
        return y + 1;
    }
    getBottomCollision(x, y) {
        while (this.collides(x, y) && y < this.height) y++;
        return y - 1;
    }

    updatePhysics() {
        this.vy += GRAVITY;
        if (this.vy > 10) this.vy = 10;

        // Горизонталь
        this.playerX += this.vx;
        if (this.collides(this.playerX, this.playerY)) {
            if (this.vx > 0) this.playerX = this.getLeftCollision(this.playerX, this.playerY);
            else if (this.vx < 0) this.playerX = this.getRightCollision(this.playerX, this.playerY);
            this.vx = 0;
        }

        // Вертикаль
        this.playerY += this.vy;
        if (this.collides(this.playerX, this.playerY)) {
            if (this.vy > 0) {
                this.playerY = this.getTopCollision(this.playerX, this.playerY);
                this.vy = 0;
                this.onGround = true;
            } else if (this.vy < 0) {
                this.playerY = this.getBottomCollision(this.playerX, this.playerY);
                this.vy = 0;
            }
        }

        // Монеты
        const cell = this.level[this.playerY]?.[this.playerX];
        if (cell === COIN) {
            this.level[this.playerY][this.playerX] = ' ';
            this.coins++;
        }

        // Выход
        if (this.playerX === this.exitX && this.playerY === this.exitY) {
            this.won = true;
            this.gameOver = true;
            const elapsed = (Date.now() - this.startTime) / 1000;
            if (this.record === null || elapsed < this.record) {
                this.record = elapsed;
                this.saveRecord(elapsed);
            }
        }
    }

    update() {
        if (this.gameOver || this.paused) return;
        this.updatePhysics();
    }

    draw() {
        console.clear();
        const elapsed = (Date.now() - this.startTime) / 1000;
        console.log('═'.repeat(this.width));
        console.log(`  Монеты: ${this.coins}/${this.totalCoins}   Время: ${elapsed.toFixed(1)} сек`);
        if (this.record !== null) console.log(`  Рекорд: ${this.record.toFixed(1)} сек`);
        console.log('═'.repeat(this.width));

        const grid = this.level.map(row => [...row]);
        grid[this.playerY][this.playerX] = PLAYER;
        grid[this.exitY][this.exitX] = EXIT;

        for (const row of grid) {
            let line = row.join('');
            line = line.replace(/\$/g, '\x1b[35m$\x1b[0m');
            line = line.replace(/#/g, '\x1b[34m#\x1b[0m');
            console.log('│' + line + '│');
        }
        console.log('═'.repeat(this.width));
        let status = this.paused ? 'ПАУЗА' : this.won ? 'ПОБЕДА!' : 'ИГРА';
        if (this.gameOver && !this.won) status = 'ИГРА ОКОНЧЕНА';
        console.log(`  ${status}  |  ← → - движение  |  Пробел/↑ - прыжок  |  P - пауза  |  R - рестарт  |  Q - выход`);
    }

    reset() {
        this.playerX = 0;
        this.playerY = 0;
        this.vx = 0;
        this.vy = 0;
        this.onGround = false;
        this.coins = 0;
        this.gameOver = false;
        this.won = false;
        this.paused = false;
        this.startTime = Date.now();
        this.level = LEVEL.map(row => [...row]);
        this.findPlayerAndExit();
    }

    handleKey(ch, key) {
        if (!key) return;
        if (key.name === 'left' || key.name === 'a') {
            if (!this.gameOver && !this.paused) this.vx = -1.5;
        } else if (key.name === 'right' || key.name === 'd') {
            if (!this.gameOver && !this.paused) this.vx = 1.5;
        } else if (key.name === 'space' || key.name === 'up') {
            this.jump();
        } else if (key.name === 'p') {
            this.paused = !this.paused;
        } else if (key.name === 'r') {
            this.reset();
        } else if (key.name === 'q') {
            this.running = false;
            process.stdin.pause();
            process.exit(0);
        }
        // Остановка движения при отпускании клавиш
        // Но keypress не даёт события отпускания, поэтому сделаем отдельный таймер
    }

    run() {
        keypress(process.stdin);
        process.stdin.setRawMode(true);
        process.stdin.resume();
        process.stdin.on('keypress', (ch, key) => this.handleKey(ch, key));

        // Таймер для сброса vx при отпускании (упрощённо)
        setInterval(() => {
            if (!this.gameOver && !this.paused) {
                // Если не нажата ни одна клавиша, vx=0 (но мы не знаем)
                // Вместо этого используем событие keypress для установки vx
                // и отдельно будем сбрасывать при отсутствии нажатий.
                // Для простоты оставим vx постоянным при удержании.
            }
        }, 50);

        const gameLoop = () => {
            if (!this.running) return;
            const now = Date.now();
            if (now - this.lastUpdate >= 1000 / 60) {
                this.update();
                this.draw();
                this.lastUpdate = now;
            }
            this.timer = setTimeout(gameLoop, 16);
        };
        gameLoop();
    }
}

const game = new Platformer();
game.run();


---

## 💻 Код на 7 языках

### 1. Python – `platformer.py`

```python
#!/usr/bin/env python3
# platformer.py - Минимальный платформер на Python

import os
import sys
import time
import json
import threading
import keyboard
from colorama import init, Fore, Style

init(autoreset=True)

# Константы
WIDTH = 40
HEIGHT = 12
GRAVITY = 0.3
JUMP_SPEED = -5.5
PLAYER = '@'
WALL = '#'
COIN = '$'
EXIT = 'X'
EMPTY = ' '
RECORD_FILE = 'platformer_record.json'

# Уровень (массив строк)
LEVEL = [
    '                                        ',
    '                                        ',
    '          #   #                         ',
    '   $      #   #     #   #              ',
    '   @      #   #  $  #   #   X          ',
    '########   ########   #############    ',
]

# Дополним уровень до HEIGHT строк (остальные пустые)
while len(LEVEL) < HEIGHT:
    LEVEL.append(' ' * WIDTH)
LEVEL = LEVEL[:HEIGHT]

class Platformer:
    def __init__(self):
        self.width = WIDTH
        self.height = HEIGHT
        self.level = [list(row.ljust(WIDTH)) for row in LEVEL]
        self.player_x = 0
        self.player_y = 0
        self.vx = 0
        self.vy = 0
        self.on_ground = False
        self.coins = 0
        self.total_coins = sum(row.count('$') for row in LEVEL)
        self.exit_x = 0
        self.exit_y = 0
        self.find_player_and_exit()
        self.game_over = False
        self.won = False
        self.paused = False
        self.running = True
        self.start_time = time.time()
        self.record = self.load_record()
        self.lock = threading.Lock()

    def find_player_and_exit(self):
        for y, row in enumerate(self.level):
            for x, ch in enumerate(row):
                if ch == PLAYER:
                    self.player_x, self.player_y = x, y
                    self.level[y][x] = EMPTY
                elif ch == EXIT:
                    self.exit_x, self.exit_y = x, y

    def load_record(self):
        try:
            with open(RECORD_FILE, 'r') as f:
                data = json.load(f)
                return data.get('record', None)
        except:
            return None

    def save_record(self, time_taken):
        with open(RECORD_FILE, 'w') as f:
            json.dump({'record': round(time_taken, 2)}, f)

    def jump(self):
        if self.on_ground and not self.game_over and not self.paused:
            self.vy = JUMP_SPEED
            self.on_ground = False

    def update_physics(self):
        # Гравитация
        self.vy += GRAVITY
        if self.vy > 10:
            self.vy = 10

        # Горизонтальное движение (с инерцией)
        self.player_x += self.vx

        # Проверка столкновений по X
        if self.collides(self.player_x, self.player_y):
            if self.vx > 0:
                self.player_x = self.get_left_collision(self.player_x, self.player_y)
            elif self.vx < 0:
                self.player_x = self.get_right_collision(self.player_x, self.player_y)
            self.vx = 0

        # Вертикальное движение
        self.player_y += self.vy

        # Проверка столкновений по Y
        if self.collides(self.player_x, self.player_y):
            if self.vy > 0:  # падение
                self.player_y = self.get_top_collision(self.player_x, self.player_y)
                self.vy = 0
                self.on_ground = True
            elif self.vy < 0:  # прыжок
                self.player_y = self.get_bottom_collision(self.player_x, self.player_y)
                self.vy = 0

        # Сбор монет
        if self.level[self.player_y][self.player_x] == COIN:
            self.level[self.player_y][self.player_x] = EMPTY
            self.coins += 1

        # Проверка выхода
        if self.player_x == self.exit_x and self.player_y == self.exit_y:
            self.won = True
            self.game_over = True
            elapsed = time.time() - self.start_time
            if self.record is None or elapsed < self.record:
                self.record = elapsed
                self.save_record(elapsed)

    def collides(self, x, y):
        if x < 0 or x >= self.width or y < 0 or y >= self.height:
            return True
        return self.level[y][x] == WALL

    # Вспомогательные для разрешения коллизий
    def get_left_collision(self, x, y):
        # Возвращает x, при котором нет коллизии
        while self.collides(x, y) and x >= 0:
            x -= 1
        return x + 1

    def get_right_collision(self, x, y):
        while self.collides(x, y) and x < self.width:
            x += 1
        return x - 1

    def get_top_collision(self, x, y):
        while self.collides(x, y) and y >= 0:
            y -= 1
        return y + 1

    def get_bottom_collision(self, x, y):
        while self.collides(x, y) and y < self.height:
            y += 1
        return y - 1

    def update(self):
        if self.game_over or self.paused:
            return
        self.update_physics()

    def draw(self):
        os.system('cls' if os.name == 'nt' else 'clear')
        print('═' * self.width)
        elapsed = time.time() - self.start_time
        print(f'  Монеты: {self.coins}/{self.total_coins}   Время: {elapsed:.1f} сек')
        if self.record is not None:
            print(f'  Рекорд: {self.record:.1f} сек')
        print('═' * self.width)

        # Копируем уровень
        grid = [row[:] for row in self.level]
        # Размещаем игрока
        grid[self.player_y][self.player_x] = Fore.GREEN + PLAYER + Style.RESET_ALL
        # Выход
        grid[self.exit_y][self.exit_x] = Fore.YELLOW + EXIT + Style.RESET_ALL
        # Монеты уже раскрашены при выводе
        for row in grid:
            line = ''.join(row)
            # Цвет для монет
            line = line.replace('$', Fore.MAGENTA + '$' + Style.RESET_ALL)
            # Стены
            line = line.replace('#', Fore.BLUE + '#' + Style.RESET_ALL)
            print('│' + line + '│')
        print('═' * self.width)
        status = "ПАУЗА" if self.paused else ("ПОБЕДА!" if self.won else "ИГРА")
        if self.game_over and not self.won:
            status = "ИГРА ОКОНЧЕНА"
        print(f'  {status}  |  ← → - движение  |  Пробел/↑ - прыжок  |  P - пауза  |  R - рестарт  |  Q - выход')

    def handle_input(self):
        while self.running:
            try:
                if keyboard.is_pressed('left') or keyboard.is_pressed('a'):
                    with self.lock:
                        if not self.game_over and not self.paused:
                            self.vx = -1.5
                elif keyboard.is_pressed('right') or keyboard.is_pressed('d'):
                    with self.lock:
                        if not self.game_over and not self.paused:
                            self.vx = 1.5
                else:
                    with self.lock:
                        if not self.game_over and not self.paused:
                            self.vx = 0
                if keyboard.is_pressed('space') or keyboard.is_pressed('up'):
                    with self.lock:
                        self.jump()
                elif keyboard.is_pressed('p'):
                    with self.lock:
                        self.paused = not self.paused
                    time.sleep(0.2)
                elif keyboard.is_pressed('r'):
                    with self.lock:
                        self.reset()
                    time.sleep(0.2)
                elif keyboard.is_pressed('q'):
                    self.running = False
                    break
            except:
                pass
            time.sleep(0.02)

    def reset(self):
        self.player_x, self.player_y = 0, 0
        self.vx = 0
        self.vy = 0
        self.on_ground = False
        self.coins = 0
        self.game_over = False
        self.won = False
        self.paused = False
        self.start_time = time.time()
        # Восстанавливаем уровень
        self.level = [list(row.ljust(WIDTH)) for row in LEVEL]
        self.find_player_and_exit()

    def run(self):
        input_thread = threading.Thread(target=self.handle_input, daemon=True)
        input_thread.start()

        last_update = time.time()
        while self.running:
            now = time.time()
            if now - last_update >= 1.0 / 60:
                with self.lock:
                    self.update()
                    self.draw()
                last_update = now
            time.sleep(0.01)

if __name__ == "__main__":
    game = Platformer()
    try:
        game.run()
    except KeyboardInterrupt:
        print("\nВыход...")

🎮 Minimal Platformer – консольный платформер

**Простой платформер с физикой** на **7 языках программирования**.  
Один уровень, управление персонажем, сбор монет и выход на финиш.  
Идеально для знакомства с основами игрового цикла, коллизий и ввода.

---

## 🎮 Геймплей

- Персонаж (`@`) передвигается по платформам (`#`).
- Цель – добраться до выхода (`X`), собирая по пути монеты (`$`).
- **Управление**: `←` / `→` – движение, `Пробел` или `↑` – прыжок.
- **Физика**: гравитация, столкновения с платформами.
- **Счёт**: количество собранных монет и затраченное время.
- Рекорд времени сохраняется в файл.

---

## 🖥️ Пример уровня
═══════════════════════════════════════
Монеты: 0 Время: 5.2 сек
═══════════════════════════════════════
│ │
│ # # │
│  # # X │
│######## ######## ##############│
═══════════════════════════════════════
← → - движение | Пробел/↑ - прыжок

text

---

## 🚀 Запуск

| Язык       | Файл          | Команда запуска                              | Зависимости                |
|------------|---------------|----------------------------------------------|----------------------------|
| Python     | `platformer.py`| `python platformer.py`                      | `keyboard`, `colorama`     |
| JavaScript | `platformer.js`| `node platformer.js`                        | `keypress`                 |
| Java       | `Platformer.java`| `javac Platformer.java && java Platformer` | `jline`, `gson`            |
| C++        | `platformer.cpp`| `g++ -std=c++17 platformer.cpp -o plat && ./plat` | (Windows: `conio` / Unix: `ncurses`) |
| C#         | `platformer.cs`| `csc platformer.cs && platformer.exe`       | .NET SDK                   |
| Go         | `platformer.go`| `go run platformer.go`                      | `keyboard`                 |
| Rust       | `platformer.rs`| `cargo run`                                 | `crossterm`, `serde_json`  |

---

## 📦 Установка зависимостей

### Python
```bash
pip install keyboard colorama
JavaScript (Node.js)
bash
npm install keypress
Java
Скачайте jline-3.21.0.jar и gson-2.8.9.jar, добавьте в classpath.

C++
Windows: conio.h встроена.

Unix/Linux: установите ncurses:

bash
sudo apt-get install libncurses-dev   # Debian/Ubuntu
sudo yum install ncurses-devel        # RHEL
Go
bash
go get github.com/eiannone/keyboard
Rust
В Cargo.toml:

toml
[dependencies]
crossterm = "0.27"
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
🛠️ Продвинутые функции
Физика – гравитация, прыжки, столкновения с платформами (сверху, сбоку, снизу).

Уровень – загружается из строкового массива, легко модифицировать.

Сбор монет – при касании монета исчезает, счёт увеличивается.

Таймер – засекает время прохождения уровня.

Рекорд – лучшее время сохраняется в JSON.

Пауза/Рестарт – по клавишам P и R.

📁 Структура репозитория
text
/
├── README.md
├── platformer.py
├── platformer.js
├── Platformer.java
├── platformer.cpp
├── platformer.cs
├── platformer.go
├── platformer.rs
└── (для Rust) Cargo.toml + src/main.rs
📜 Лицензия
MIT – свободно используйте, улучшайте и распространяйте.


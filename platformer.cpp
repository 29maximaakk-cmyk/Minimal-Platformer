// platformer.cpp - Минимальный платформер на C++17
#include <iostream>
#include <vector>
#include <string>
#include <fstream>
#include <chrono>
#include <thread>
#include <cstdlib>
#include <algorithm>
#include <random>

#ifdef _WIN32
    #include <conio.h>
    #include <windows.h>
    #define CLEAR() system("cls")
#else
    #include <ncurses.h>
    #include <termios.h>
    #include <unistd.h>
    #include <fcntl.h>
    #define CLEAR() system("clear")
#endif

const int WIDTH = 40;
const int HEIGHT = 12;
const double GRAVITY = 0.3;
const double JUMP_SPEED = -5.5;
const char PLAYER = '@';
const char WALL = '#';
const char COIN = '$';
const char EXIT = 'X';
const std::string RECORD_FILE = "platformer_record.json";

std::vector<std::string> LEVEL_RAW = {
    "                                        ",
    "                                        ",
    "          #   #                         ",
    "   $      #   #     #   #              ",
    "   @      #   #  $  #   #   X          ",
    "########   ########   #############    ",
};
while (LEVEL_RAW.size() < HEIGHT) LEVEL_RAW.push_back(std::string(WIDTH, ' '));

class Platformer {
public:
    Platformer() : running(true), record(loadRecord()) {
        reset();
    }

    ~Platformer() {
#ifndef _WIN32
        endwin();
#endif
    }

    void run() {
#ifdef _WIN32
        auto lastUpdate = std::chrono::steady_clock::now();
        while (running) {
            handleInputWindows();
            auto now = std::chrono::steady_clock::now();
            if (std::chrono::duration_cast<std::chrono::milliseconds>(now - lastUpdate).count() >= 1000/60) {
                update();
                draw();
                lastUpdate = now;
            }
            std::this_thread::sleep_for(std::chrono::milliseconds(10));
        }
#else
        initscr();
        raw();
        noecho();
        keypad(stdscr, TRUE);
        nodelay(stdscr, TRUE);
        curs_set(0);
        auto lastUpdate = std::chrono::steady_clock::now();
        while (running) {
            handleInputNcurses();
            auto now = std::chrono::steady_clock::now();
            if (std::chrono::duration_cast<std::chrono::milliseconds>(now - lastUpdate).count() >= 1000/60) {
                update();
                drawNcurses();
                lastUpdate = now;
            }
            napms(10);
        }
        endwin();
#endif
    }

private:
    std::vector<std::string> level;
    int playerX, playerY;
    double vx, vy;
    bool onGround;
    int coins, totalCoins;
    int exitX, exitY;
    bool gameOver, won, paused, running;
    std::chrono::steady_clock::time_point startTime;
    double record;
    std::mt19937 rng;

    double loadRecord() {
        std::ifstream in(RECORD_FILE);
        if (!in) return -1.0;
        std::string content((std::istreambuf_iterator<char>(in)), std::istreambuf_iterator<char>());
        size_t pos = content.find("\"record\":");
        if (pos != std::string::npos) {
            pos += 9;
            size_t end = content.find(",", pos);
            if (end == std::string::npos) end = content.find("}", pos);
            return std::stod(content.substr(pos, end - pos));
        }
        return -1.0;
    }

    void saveRecord(double time) {
        std::ofstream out(RECORD_FILE);
        out << "{\"record\":" << time << "}";
    }

    void findPlayerAndExit() {
        for (int y = 0; y < (int)level.size(); y++) {
            for (int x = 0; x < (int)level[y].size(); x++) {
                char ch = level[y][x];
                if (ch == PLAYER) {
                    playerX = x; playerY = y;
                    level[y][x] = ' ';
                } else if (ch == EXIT) {
                    exitX = x; exitY = y;
                } else if (ch == COIN) {
                    totalCoins++;
                }
            }
        }
    }

    void reset() {
        level = LEVEL_RAW;
        playerX = playerY = 0;
        vx = vy = 0;
        onGround = false;
        coins = 0;
        totalCoins = 0;
        gameOver = false;
        won = false;
        paused = false;
        startTime = std::chrono::steady_clock::now();
        findPlayerAndExit();
    }

    bool collides(int x, int y) {
        if (x < 0 || x >= WIDTH || y < 0 || y >= HEIGHT) return true;
        return level[y][x] == WALL;
    }

    int getLeftCollision(int x, int y) {
        while (collides(x, y) && x >= 0) x--;
        return x + 1;
    }
    int getRightCollision(int x, int y) {
        while (collides(x, y) && x < WIDTH) x++;
        return x - 1;
    }
    int getTopCollision(int x, int y) {
        while (collides(x, y) && y >= 0) y--;
        return y + 1;
    }
    int getBottomCollision(int x, int y) {
        while (collides(x, y) && y < HEIGHT) y++;
        return y - 1;
    }

    void updatePhysics() {
        vy += GRAVITY;
        if (vy > 10) vy = 10;

        playerX += vx;
        if (collides(playerX, playerY)) {
            if (vx > 0) playerX = getLeftCollision(playerX, playerY);
            else if (vx < 0) playerX = getRightCollision(playerX, playerY);
            vx = 0;
        }

        playerY += vy;
        if (collides(playerX, playerY)) {
            if (vy > 0) {
                playerY = getTopCollision(playerX, playerY);
                vy = 0;
                onGround = true;
            } else if (vy < 0) {
                playerY = getBottomCollision(playerX, playerY);
                vy = 0;
            }
        }

        if (level[playerY][playerX] == COIN) {
            level[playerY][playerX] = ' ';
            coins++;
        }

        if (playerX == exitX && playerY == exitY) {
            won = true;
            gameOver = true;
            auto elapsed = std::chrono::duration<double>(std::chrono::steady_clock::now() - startTime).count();
            if (record < 0 || elapsed < record) {
                record = elapsed;
                saveRecord(elapsed);
            }
        }
    }

    void update() {
        if (gameOver || paused) return;
        updatePhysics();
    }

    void draw() {
        CLEAR();
        auto elapsed = std::chrono::duration<double>(std::chrono::steady_clock::now() - startTime).count();
        std::cout << std::string(WIDTH, '═') << std::endl;
        printf("  Монеты: %d/%d   Время: %.1f сек\n", coins, totalCoins, elapsed);
        if (record >= 0) printf("  Рекорд: %.1f сек\n", record);
        std::cout << std::string(WIDTH, '═') << std::endl;

        auto grid = level;
        grid[playerY][playerX] = PLAYER;
        grid[exitY][exitX] = EXIT;

        for (int y = 0; y < HEIGHT; y++) {
            std::cout << '│';
            for (int x = 0; x < WIDTH; x++) {
                char ch = grid[y][x];
                if (ch == COIN) std::cout << "\033[35m$\033[0m";
                else if (ch == WALL) std::cout << "\033[34m#\033[0m";
                else if (ch == PLAYER) std::cout << "\033[32m@\033[0m";
                else if (ch == EXIT) std::cout << "\033[33mX\033[0m";
                else std::cout << ch;
            }
            std::cout << '│' << std::endl;
        }
        std::cout << std::string(WIDTH, '═') << std::endl;
        std::string status = paused ? "ПАУЗА" : won ? "ПОБЕДА!" : gameOver ? "ИГРА ОКОНЧЕНА" : "ИГРА";
        std::cout << "  " << status << "  |  ← → - движение  |  Пробел/↑ - прыжок  |  P - пауза  |  R - рестарт  |  Q - выход" << std::endl;
    }

#ifdef _WIN32
    void handleInputWindows() {
        if (_kbhit()) {
            int ch = _getch();
            if (ch == 224) { // стрелки
                ch = _getch();
                if (ch == 75) { // left
                    if (!gameOver && !paused) vx = -1.5;
                } else if (ch == 77) { // right
                    if (!gameOver && !paused) vx = 1.5;
                } else if (ch == 72) { // up
                    jump();
                }
            } else {
                switch (tolower(ch)) {
                    case 'a': if (!gameOver && !paused) vx = -1.5; break;
                    case 'd': if (!gameOver && !paused) vx = 1.5; break;
                    case ' ': jump(); break;
                    case 'p': paused = !paused; break;
                    case 'r': reset(); break;
                    case 'q': running = false; break;
                }
            }
        } else {
            if (!gameOver && !paused) vx = 0; // отпускание
        }
    }
#else
    void handleInputNcurses() {
        int ch = getch();
        if (ch == ERR) {
            if (!gameOver && !paused) vx = 0;
            return;
        }
        switch (ch) {
            case KEY_LEFT:
            case 'a': case 'A':
                if (!gameOver && !paused) vx = -1.5; break;
            case KEY_RIGHT:
            case 'd': case 'D':
                if (!gameOver && !paused) vx = 1.5; break;
            case ' ':
            case KEY_UP:
                jump(); break;
            case 'p': case 'P': paused = !paused; break;
            case 'r': case 'R': reset(); break;
            case 'q': case 'Q': running = false; break;
        }
    }
#endif

    void jump() {
        if (onGround && !gameOver && !paused) {
            vy = JUMP_SPEED;
            onGround = false;
        }
    }
};

int main() {
    Platformer game;
    game.run();
    return 0;
}

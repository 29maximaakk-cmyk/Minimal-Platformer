// Platformer.java - Минимальный платформер на Java
import org.jline.terminal.Terminal;
import org.jline.terminal.TerminalBuilder;
import org.jline.utils.InfoCmp;
import com.google.gson.*;

import java.io.*;
import java.nio.file.*;
import java.util.*;

public class Platformer {
    private static final int WIDTH = 40;
    private static final int HEIGHT = 12;
    private static final double GRAVITY = 0.3;
    private static final double JUMP_SPEED = -5.5;
    private static final char PLAYER = '@';
    private static final char WALL = '#';
    private static final char COIN = '$';
    private static final char EXIT = 'X';
    private static final String RECORD_FILE = "platformer_record.json";

    private static final String[] LEVEL_RAW = {
        "                                        ",
        "                                        ",
        "          #   #                         ",
        "   $      #   #     #   #              ",
        "   @      #   #  $  #   #   X          ",
        "########   ########   #############    ",
    };
    private static char[][] LEVEL;

    static {
        int rows = Math.max(LEVEL_RAW.length, HEIGHT);
        LEVEL = new char[rows][WIDTH];
        for (int i = 0; i < rows; i++) {
            String row = i < LEVEL_RAW.length ? LEVEL_RAW[i] : " ".repeat(WIDTH);
            for (int j = 0; j < WIDTH; j++) {
                LEVEL[i][j] = j < row.length() ? row.charAt(j) : ' ';
            }
        }
    }

    private Terminal terminal;
    private char[][] level;
    private int playerX, playerY;
    private double vx, vy;
    private boolean onGround;
    private int coins, totalCoins;
    private int exitX, exitY;
    private boolean gameOver, won, paused, running;
    private long startTime;
    private Double record;
    private long lastUpdate;

    public Platformer() throws IOException {
        terminal = TerminalBuilder.builder().system(true).build();
        resetState();
        record = loadRecord();
        lastUpdate = System.currentTimeMillis();
    }

    private void resetState() {
        level = copyLevel(LEVEL);
        playerX = playerY = 0;
        vx = vy = 0;
        onGround = false;
        coins = 0;
        totalCoins = 0;
        gameOver = false;
        won = false;
        paused = false;
        running = true;
        startTime = System.currentTimeMillis();
        findPlayerAndExit();
    }

    private char[][] copyLevel(char[][] src) {
        char[][] dst = new char[src.length][];
        for (int i = 0; i < src.length; i++) {
            dst[i] = Arrays.copyOf(src[i], src[i].length);
        }
        return dst;
    }

    private void findPlayerAndExit() {
        for (int y = 0; y < level.length; y++) {
            for (int x = 0; x < level[y].length; x++) {
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

    private Double loadRecord() {
        try {
            String content = new String(Files.readAllBytes(Paths.get(RECORD_FILE)));
            JsonObject obj = new Gson().fromJson(content, JsonObject.class);
            return obj.get("record").getAsDouble();
        } catch (Exception e) { return null; }
    }

    private void saveRecord(double time) {
        try {
            JsonObject obj = new JsonObject();
            obj.addProperty("record", time);
            Files.write(Paths.get(RECORD_FILE), obj.toString().getBytes());
        } catch (Exception e) {}
    }

    private void jump() {
        if (onGround && !gameOver && !paused) {
            vy = JUMP_SPEED;
            onGround = false;
        }
    }

    private boolean collides(int x, int y) {
        if (x < 0 || x >= WIDTH || y < 0 || y >= HEIGHT) return true;
        return level[y][x] == WALL;
    }

    private int getLeftCollision(int x, int y) {
        while (collides(x, y) && x >= 0) x--;
        return x + 1;
    }
    private int getRightCollision(int x, int y) {
        while (collides(x, y) && x < WIDTH) x++;
        return x - 1;
    }
    private int getTopCollision(int x, int y) {
        while (collides(x, y) && y >= 0) y--;
        return y + 1;
    }
    private int getBottomCollision(int x, int y) {
        while (collides(x, y) && y < HEIGHT) y++;
        return y - 1;
    }

    private void updatePhysics() {
        vy += GRAVITY;
        if (vy > 10) vy = 10;

        // Горизонталь
        playerX += vx;
        if (collides(playerX, playerY)) {
            if (vx > 0) playerX = getLeftCollision(playerX, playerY);
            else if (vx < 0) playerX = getRightCollision(playerX, playerY);
            vx = 0;
        }

        // Вертикаль
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

        // Монеты
        if (playerY >= 0 && playerY < level.length && playerX >= 0 && playerX < level[playerY].length) {
            if (level[playerY][playerX] == COIN) {
                level[playerY][playerX] = ' ';
                coins++;
            }
        }

        // Выход
        if (playerX == exitX && playerY == exitY) {
            won = true;
            gameOver = true;
            double elapsed = (System.currentTimeMillis() - startTime) / 1000.0;
            if (record == null || elapsed < record) {
                record = elapsed;
                saveRecord(elapsed);
            }
        }
    }

    private void update() {
        if (gameOver || paused) return;
        updatePhysics();
    }

    private void draw() {
        terminal.puts(InfoCmp.Capability.clear_screen);
        double elapsed = (System.currentTimeMillis() - startTime) / 1000.0;
        System.out.println("═".repeat(WIDTH));
        System.out.printf("  Монеты: %d/%d   Время: %.1f сек%n", coins, totalCoins, elapsed);
        if (record != null) System.out.printf("  Рекорд: %.1f сек%n", record);
        System.out.println("═".repeat(WIDTH));

        char[][] grid = copyLevel(level);
        grid[playerY][playerX] = PLAYER;
        grid[exitY][exitX] = EXIT;

        for (int y = 0; y < grid.length; y++) {
            System.out.print('│');
            for (int x = 0; x < grid[y].length; x++) {
                char ch = grid[y][x];
                if (ch == COIN) System.out.print("\033[35m$\033[0m");
                else if (ch == WALL) System.out.print("\033[34m#\033[0m");
                else if (ch == PLAYER) System.out.print("\033[32m@\033[0m");
                else if (ch == EXIT) System.out.print("\033[33mX\033[0m");
                else System.out.print(ch);
            }
            System.out.println('│');
        }
        System.out.println("═".repeat(WIDTH));
        String status = paused ? "ПАУЗА" : won ? "ПОБЕДА!" : gameOver ? "ИГРА ОКОНЧЕНА" : "ИГРА";
        System.out.printf("  %s  |  ← → - движение  |  Пробел/↑ - прыжок  |  P - пауза  |  R - рестарт  |  Q - выход%n", status);
    }

    private void reset() {
        resetState();
        findPlayerAndExit();
        startTime = System.currentTimeMillis();
    }

    private void handleInput() {
        try {
            while (running) {
                int ch = terminal.reader().read();
                if (ch == -1) continue;
                char c = (char) ch;
                switch (c) {
                    case 'a': case 'A':
                        if (!gameOver && !paused) vx = -1.5; break;
                    case 'd': case 'D':
                        if (!gameOver && !paused) vx = 1.5; break;
                    case ' ':
                    case '↑':
                        jump(); break;
                    case 'p': case 'P': paused = !paused; break;
                    case 'r': case 'R': reset(); break;
                    case 'q': case 'Q': running = false; break;
                    default:
                        if (!gameOver && !paused) vx = 0; // отпускание клавиш
                }
                // Для стрелок нужно ловить ESC-последовательности, упростим
            }
        } catch (IOException e) {}
    }

    public void run() throws Exception {
        Thread inputThread = new Thread(this::handleInput);
        inputThread.setDaemon(true);
        inputThread.start();

        while (running) {
            long now = System.currentTimeMillis();
            if (now - lastUpdate >= 1000 / 60) {
                update();
                draw();
                lastUpdate = now;
            }
            Thread.sleep(16);
        }
        terminal.close();
    }

    public static void main(String[] args) throws Exception {
        Platformer game = new Platformer();
        game.run();
    }
}

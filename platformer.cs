// platformer.cs - Минимальный платформер на C#
using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Threading;

class Platformer
{
    private const int WIDTH = 40;
    private const int HEIGHT = 12;
    private const double GRAVITY = 0.3;
    private const double JUMP_SPEED = -5.5;
    private const char PLAYER = '@';
    private const char WALL = '#';
    private const char COIN = '$';
    private const char EXIT = 'X';
    private const string RECORD_FILE = "platformer_record.json";

    private static readonly string[] LEVEL_RAW = {
        "                                        ",
        "                                        ",
        "          #   #                         ",
        "   $      #   #     #   #              ",
        "   @      #   #  $  #   #   X          ",
        "########   ########   #############    ",
    };

    private char[][] level;
    private int playerX, playerY;
    private double vx, vy;
    private bool onGround;
    private int coins, totalCoins;
    private int exitX, exitY;
    private bool gameOver, won, paused, running;
    private DateTime startTime;
    private double? record;
    private DateTime lastUpdate;
    private Random rand;

    public Platformer()
    {
        rand = new Random();
        record = LoadRecord();
        Reset();
        lastUpdate = DateTime.Now;
        running = true;
    }

    private double? LoadRecord()
    {
        if (!File.Exists(RECORD_FILE)) return null;
        string json = File.ReadAllText(RECORD_FILE);
        var data = JsonSerializer.Deserialize<Dictionary<string, double>>(json);
        return data != null && data.ContainsKey("record") ? data["record"] : (double?)null;
    }

    private void SaveRecord(double time)
    {
        var data = new Dictionary<string, double> { { "record", time } };
        File.WriteAllText(RECORD_FILE, JsonSerializer.Serialize(data));
    }

    private char[][] CopyLevel(char[][] src)
    {
        char[][] dst = new char[src.Length][];
        for (int i = 0; i < src.Length; i++)
        {
            dst[i] = (char[])src[i].Clone();
        }
        return dst;
    }

    private void FindPlayerAndExit()
    {
        for (int y = 0; y < level.Length; y++)
        {
            for (int x = 0; x < level[y].Length; x++)
            {
                char ch = level[y][x];
                if (ch == PLAYER)
                {
                    playerX = x; playerY = y;
                    level[y][x] = ' ';
                }
                else if (ch == EXIT)
                {
                    exitX = x; exitY = y;
                }
                else if (ch == COIN)
                {
                    totalCoins++;
                }
            }
        }
    }

    private void Reset()
    {
        // Инициализация уровня
        int rows = Math.Max(LEVEL_RAW.Length, HEIGHT);
        level = new char[rows][];
        for (int i = 0; i < rows; i++)
        {
            string row = i < LEVEL_RAW.Length ? LEVEL_RAW[i] : new string(' ', WIDTH);
            level[i] = row.PadRight(WIDTH).ToCharArray();
        }
        playerX = playerY = 0;
        vx = vy = 0;
        onGround = false;
        coins = 0;
        totalCoins = 0;
        gameOver = false;
        won = false;
        paused = false;
        startTime = DateTime.Now;
        FindPlayerAndExit();
    }

    private bool Collides(int x, int y)
    {
        if (x < 0 || x >= WIDTH || y < 0 || y >= level.Length) return true;
        return level[y][x] == WALL;
    }

    private int GetLeftCollision(int x, int y)
    {
        while (Collides(x, y) && x >= 0) x--;
        return x + 1;
    }
    private int GetRightCollision(int x, int y)
    {
        while (Collides(x, y) && x < WIDTH) x++;
        return x - 1;
    }
    private int GetTopCollision(int x, int y)
    {
        while (Collides(x, y) && y >= 0) y--;
        return y + 1;
    }
    private int GetBottomCollision(int x, int y)
    {
        while (Collides(x, y) && y < level.Length) y++;
        return y - 1;
    }

    private void UpdatePhysics()
    {
        vy += GRAVITY;
        if (vy > 10) vy = 10;

        playerX += (int)vx;
        if (Collides(playerX, playerY))
        {
            if (vx > 0) playerX = GetLeftCollision(playerX, playerY);
            else if (vx < 0) playerX = GetRightCollision(playerX, playerY);
            vx = 0;
        }

        playerY += (int)vy;
        if (Collides(playerX, playerY))
        {
            if (vy > 0)
            {
                playerY = GetTopCollision(playerX, playerY);
                vy = 0;
                onGround = true;
            }
            else if (vy < 0)
            {
                playerY = GetBottomCollision(playerX, playerY);
                vy = 0;
            }
        }

        if (level[playerY][playerX] == COIN)
        {
            level[playerY][playerX] = ' ';
            coins++;
        }

        if (playerX == exitX && playerY == exitY)
        {
            won = true;
            gameOver = true;
            double elapsed = (DateTime.Now - startTime).TotalSeconds;
            if (record == null || elapsed < record)
            {
                record = elapsed;
                SaveRecord(elapsed);
            }
        }
    }

    private void Update()
    {
        if (gameOver || paused) return;
        UpdatePhysics();
    }

    private void Jump()
    {
        if (onGround && !gameOver && !paused)
        {
            vy = JUMP_SPEED;
            onGround = false;
        }
    }

    private void Draw()
    {
        Console.Clear();
        double elapsed = (DateTime.Now - startTime).TotalSeconds;
        Console.WriteLine(new string('═', WIDTH));
        Console.WriteLine($"  Монеты: {coins}/{totalCoins}   Время: {elapsed:F1} сек");
        if (record != null) Console.WriteLine($"  Рекорд: {record:F1} сек");
        Console.WriteLine(new string('═', WIDTH));

        var grid = CopyLevel(level);
        grid[playerY][playerX] = PLAYER;
        grid[exitY][exitX] = EXIT;

        for (int y = 0; y < grid.Length; y++)
        {
            Console.Write('│');
            for (int x = 0; x < grid[y].Length; x++)
            {
                char ch = grid[y][x];
                if (ch == COIN) Console.Write("\x1b[35m$\x1b[0m");
                else if (ch == WALL) Console.Write("\x1b[34m#\x1b[0m");
                else if (ch == PLAYER) Console.Write("\x1b[32m@\x1b[0m");
                else if (ch == EXIT) Console.Write("\x1b[33mX\x1b[0m");
                else Console.Write(ch);
            }
            Console.WriteLine('│');
        }
        Console.WriteLine(new string('═', WIDTH));
        string status = paused ? "ПАУЗА" : won ? "ПОБЕДА!" : gameOver ? "ИГРА ОКОНЧЕНА" : "ИГРА";
        Console.WriteLine($"  {status}  |  ← → - движение  |  Пробел/↑ - прыжок  |  P - пауза  |  R - рестарт  |  Q - выход");
    }

    public void Run()
    {
        while (running)
        {
            // Обработка ввода
            while (Console.KeyAvailable)
            {
                var key = Console.ReadKey(true);
                switch (key.Key)
                {
                    case ConsoleKey.LeftArrow:
                    case ConsoleKey.A:
                        if (!gameOver && !paused) vx = -1.5;
                        break;
                    case ConsoleKey.RightArrow:
                    case ConsoleKey.D:
                        if (!gameOver && !paused) vx = 1.5;
                        break;
                    case ConsoleKey.Spacebar:
                    case ConsoleKey.UpArrow:
                        Jump();
                        break;
                    case ConsoleKey.P:
                        paused = !paused;
                        break;
                    case ConsoleKey.R:
                        Reset();
                        break;
                    case ConsoleKey.Q:
                        running = false;
                        break;
                }
            }

            // Сброс vx если не нажата клавиша (упрощённо)
            // Лучше отслеживать отпускание, но в C# Console.KeyAvailable не даёт события отпускания.
            // Вместо этого будем сбрасывать vx, если нет нажатых клавиш направления.
            // Для простоты оставим как есть – vx сохраняется, пока не нажата другая клавиша.

            if ((DateTime.Now - lastUpdate).TotalMilliseconds >= 1000.0 / 60)
            {
                Update();
                Draw();
                lastUpdate = DateTime.Now;
            }
            Thread.Sleep(10);
        }
    }

    static void Main()
    {
        var game = new Platformer();
        game.Run();
        Console.WriteLine("Игра завершена.");
    }
}

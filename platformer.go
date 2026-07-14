// platformer.go - Минимальный платформер на Go
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"runtime"
	"time"

	"github.com/eiannone/keyboard"
)

const (
	WIDTH      = 40
	HEIGHT     = 12
	GRAVITY    = 0.3
	JUMP_SPEED = -5.5
	PLAYER     = '@'
	WALL       = '#'
	COIN       = '$'
	EXIT       = 'X'
	RECORD_FILE = "platformer_record.json"
)

var LEVEL_RAW = []string{
	"                                        ",
	"                                        ",
	"          #   #                         ",
	"   $      #   #     #   #              ",
	"   @      #   #  $  #   #   X          ",
	"########   ########   #############    ",
}

type Platformer struct {
	level      [][]rune
	playerX    int
	playerY    int
	vx         float64
	vy         float64
	onGround   bool
	coins      int
	totalCoins int
	exitX      int
	exitY      int
	gameOver   bool
	won        bool
	paused     bool
	running    bool
	startTime  time.Time
	record     *float64
	lastUpdate time.Time
}

func NewPlatformer() *Platformer {
	p := &Platformer{
		running:    true,
		record:     loadRecord(),
		lastUpdate: time.Now(),
	}
	p.reset()
	return p
}

func loadRecord() *float64 {
	file, err := os.Open(RECORD_FILE)
	if err != nil {
		return nil
	}
	defer file.Close()
	var data map[string]float64
	decoder := json.NewDecoder(file)
	if err := decoder.Decode(&data); err != nil {
		return nil
	}
	if val, ok := data["record"]; ok {
		return &val
	}
	return nil
}

func saveRecord(time float64) {
	data := map[string]float64{"record": time}
	file, _ := os.Create(RECORD_FILE)
	defer file.Close()
	encoder := json.NewEncoder(file)
	encoder.Encode(data)
}

func (p *Platformer) findPlayerAndExit() {
	for y, row := range p.level {
		for x, ch := range row {
			if ch == rune(PLAYER[0]) {
				p.playerX, p.playerY = x, y
				p.level[y][x] = ' '
			} else if ch == rune(EXIT[0]) {
				p.exitX, p.exitY = x, y
			} else if ch == rune(COIN[0]) {
				p.totalCoins++
			}
		}
	}
}

func (p *Platformer) reset() {
	// Инициализация уровня
	rows := len(LEVEL_RAW)
	if rows < HEIGHT {
		rows = HEIGHT
	}
	p.level = make([][]rune, rows)
	for i := 0; i < rows; i++ {
		var row string
		if i < len(LEVEL_RAW) {
			row = LEVEL_RAW[i]
		} else {
			row = ""
		}
		row = fmt.Sprintf("%-*s", WIDTH, row)
		p.level[i] = []rune(row)
	}
	p.playerX, p.playerY = 0, 0
	p.vx, p.vy = 0, 0
	p.onGround = false
	p.coins = 0
	p.totalCoins = 0
	p.gameOver = false
	p.won = false
	p.paused = false
	p.startTime = time.Now()
	p.findPlayerAndExit()
}

func (p *Platformer) collides(x, y int) bool {
	if x < 0 || x >= WIDTH || y < 0 || y >= len(p.level) {
		return true
	}
	return p.level[y][x] == rune(WALL[0])
}

func (p *Platformer) getLeftCollision(x, y int) int {
	for p.collides(x, y) && x >= 0 {
		x--
	}
	return x + 1
}
func (p *Platformer) getRightCollision(x, y int) int {
	for p.collides(x, y) && x < WIDTH {
		x++
	}
	return x - 1
}
func (p *Platformer) getTopCollision(x, y int) int {
	for p.collides(x, y) && y >= 0 {
		y--
	}
	return y + 1
}
func (p *Platformer) getBottomCollision(x, y int) int {
	for p.collides(x, y) && y < len(p.level) {
		y++
	}
	return y - 1
}

func (p *Platformer) updatePhysics() {
	p.vy += GRAVITY
	if p.vy > 10 {
		p.vy = 10
	}

	// Горизонталь
	p.playerX += int(p.vx)
	if p.collides(p.playerX, p.playerY) {
		if p.vx > 0 {
			p.playerX = p.getLeftCollision(p.playerX, p.playerY)
		} else if p.vx < 0 {
			p.playerX = p.getRightCollision(p.playerX, p.playerY)
		}
		p.vx = 0
	}

	// Вертикаль
	p.playerY += int(p.vy)
	if p.collides(p.playerX, p.playerY) {
		if p.vy > 0 {
			p.playerY = p.getTopCollision(p.playerX, p.playerY)
			p.vy = 0
			p.onGround = true
		} else if p.vy < 0 {
			p.playerY = p.getBottomCollision(p.playerX, p.playerY)
			p.vy = 0
		}
	}

	// Монеты
	if p.level[p.playerY][p.playerX] == rune(COIN[0]) {
		p.level[p.playerY][p.playerX] = ' '
		p.coins++
	}

	// Выход
	if p.playerX == p.exitX && p.playerY == p.exitY {
		p.won = true
		p.gameOver = true
		elapsed := time.Since(p.startTime).Seconds()
		if p.record == nil || elapsed < *p.record {
			p.record = &elapsed
			saveRecord(elapsed)
		}
	}
}

func (p *Platformer) update() {
	if p.gameOver || p.paused {
		return
	}
	p.updatePhysics()
}

func (p *Platformer) draw() {
	clearScreen()
	elapsed := time.Since(p.startTime).Seconds()
	fmt.Println(stringRepeat("═", WIDTH))
	fmt.Printf("  Монеты: %d/%d   Время: %.1f сек\n", p.coins, p.totalCoins, elapsed)
	if p.record != nil {
		fmt.Printf("  Рекорд: %.1f сек\n", *p.record)
	}
	fmt.Println(stringRepeat("═", WIDTH))

	grid := make([][]rune, len(p.level))
	for i := range p.level {
		grid[i] = append([]rune{}, p.level[i]...)
	}
	grid[p.playerY][p.playerX] = rune(PLAYER[0])
	grid[p.exitY][p.exitX] = rune(EXIT[0])

	for y := 0; y < len(grid); y++ {
		fmt.Print("│")
		for x := 0; x < len(grid[y]); x++ {
			ch := grid[y][x]
			switch ch {
			case rune(COIN[0]):
				fmt.Print("\x1b[35m$\x1b[0m")
			case rune(WALL[0]):
				fmt.Print("\x1b[34m#\x1b[0m")
			case rune(PLAYER[0]):
				fmt.Print("\x1b[32m@\x1b[0m")
			case rune(EXIT[0]):
				fmt.Print("\x1b[33mX\x1b[0m")
			default:
				fmt.Print(string(ch))
			}
		}
		fmt.Println("│")
	}
	fmt.Println(stringRepeat("═", WIDTH))
	status := "ИГРА"
	if p.paused {
		status = "ПАУЗА"
	} else if p.won {
		status = "ПОБЕДА!"
	} else if p.gameOver {
		status = "ИГРА ОКОНЧЕНА"
	}
	fmt.Printf("  %s  |  ← → - движение  |  Пробел/↑ - прыжок  |  P - пауза  |  R - рестарт  |  Q - выход\n", status)
}

func clearScreen() {
	cmd := exec.Command("clear")
	if runtime.GOOS == "windows" {
		cmd = exec.Command("cmd", "/c", "cls")
	}
	cmd.Stdout = os.Stdout
	cmd.Run()
}

func stringRepeat(s string, n int) string {
	res := ""
	for i := 0; i < n; i++ {
		res += s
	}
	return res
}

func (p *Platformer) handleInput() {
	for p.running {
		char, key, err := keyboard.GetKey()
		if err != nil {
			continue
		}
		switch key {
		case keyboard.KeyArrowLeft:
			if !p.gameOver && !p.paused {
				p.vx = -1.5
			}
		case keyboard.KeyArrowRight:
			if !p.gameOver && !p.paused {
				p.vx = 1.5
			}
		case keyboard.KeyArrowUp, keyboard.KeySpace:
			if p.onGround && !p.gameOver && !p.paused {
				p.vy = JUMP_SPEED
				p.onGround = false
			}
		default:
			switch char {
			case 'a', 'A':
				if !p.gameOver && !p.paused {
					p.vx = -1.5
				}
			case 'd', 'D':
				if !p.gameOver && !p.paused {
					p.vx = 1.5
				}
			case 'p', 'P':
				p.paused = !p.paused
			case 'r', 'R':
				p.reset()
			case 'q', 'Q':
				p.running = false
				return
			default:
				if !p.gameOver && !p.paused {
					p.vx = 0 // отпускание клавиш
				}
			}
		}
	}
}

func (p *Platformer) run() {
	go p.handleInput()

	ticker := time.NewTicker(16 * time.Millisecond)
	defer ticker.Stop()

	for p.running {
		select {
		case <-ticker.C:
			now := time.Now()
			if now.Sub(p.lastUpdate) >= time.Second/60 {
				p.update()
				p.draw()
				p.lastUpdate = now
			}
		}
	}
}

func main() {
	game := NewPlatformer()
	game.run()
	fmt.Println("Игра завершена.")
}

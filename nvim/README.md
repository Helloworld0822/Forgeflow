# AutoForge Neovim (Arduino + STM32)

임베디드 개발용 Neovim 설정입니다.

- **Arduino (`.ino`)** — [yuukiflow/Arduino-Nvim](https://github.com/yuukiflow/Arduino-Nvim) (LSP, 보드/포트 GUI, 라이브러리 관리)
- **STM32 / PlatformIO** — `autoforge-mcu` (빌드, 플래시, 시리얼 모니터)

## 역할 분담

| 대상 | 플러그인 | 키맵 prefix |
|------|----------|-------------|
| Arduino `.ino` | Arduino-Nvim | `<leader>a` (`ac` 빌드, `au` 업로드, `am` 모니터) |
| STM32 / PlatformIO | autoforge-mcu | `<leader>m` (`mb` 빌드, `mu` 업로드, `mm` 모니터) |

Arduino 프로젝트에서 `:McuBuild` 등을 호출하면 자동으로 Arduino-Nvim (`:InoCheck` 등)으로 위임됩니다.

## 필요 도구

**Arduino (Arduino-Nvim)**

```bash
sudo pacman -S arduino-cli clang
go install github.com/arduino/arduino-language-server@latest
arduino-cli core update-index
arduino-cli core install arduino:avr
```

**STM32 / PlatformIO (autoforge-mcu)**

```bash
pip install platformio
sudo pacman -S stlink openocd
```

## 설치 (lazy.nvim, 권장)

`~/.config/nvim/init.lua`:

```lua
local autoforge_nvim = vim.fn.expand("~/code/AutoForge/nvim")
vim.opt.rtp:prepend(autoforge_nvim)

require("autoforge-nvim").setup({
  arduino_nvim = {
    board = "arduino:avr:uno",
    port = "/dev/ttyACM0",
    baudrate = 115200,
    picker_backend = "telescope", -- telescope | nvim
  },
  autoforge_mcu = {
    arduino_backend = "arduino-nvim",
    map_prefix = "<leader>m",
  },
})

require("lazy").setup(require("plugins.init"), {
  root = vim.fn.stdpath("data") .. "/lazy/autoforge",
})
```

또는 한 번에 부트스트랩:

```bash
nvim -u ~/code/AutoForge/nvim/lazy-setup.lua
```

### 기존 lazy.nvim 설정에 플러그인만 추가

`lua/plugins/autoforge.lua`:

```lua
local autoforge_nvim = vim.fn.expand("~/code/AutoForge/nvim")
vim.opt.rtp:prepend(autoforge_nvim)
require("autoforge-nvim").setup({})

return require("plugins.init")
```

## Arduino-Nvim 명령어 / 키맵

| 키맵 | 명령어 | 설명 |
|------|--------|------|
| `<leader>ac` | `:InoCheck` | 컴파일·검증 |
| `<leader>au` | `:InoUpload` | 업로드 |
| `<leader>ar` | `:InoUploadReset` | 리셋 업로드 (UNO R4 WiFi 등) |
| `<leader>am` | `:InoMonitor` | 시리얼 모니터 |
| `<leader>as` | `:InoStatus` | 보드/포트/FQBN 상태 |
| `<leader>al` | `:InoLib` | 라이브러리 관리 (Telescope) |
| `<leader>ag` | `:InoGUI` | 보드·포트 GUI |
| `<leader>ap` | `:InoSelectPort` | 포트 선택 |
| `<leader>ab` | `:InoSelectBoard` | 보드 선택 |

프로젝트 루트에 `.arduino_config.lua`가 자동 생성됩니다.

## autoforge-mcu 명령어 (STM32 / PlatformIO)

| 명령어 | 설명 |
|--------|------|
| `:McuInfo` | 프로젝트 타입·루트 |
| `:McuBuild` | 빌드 |
| `:McuUpload` | 업로드/플래시 |
| `:McuBuildUpload` | 빌드 후 업로드 |
| `:McuMonitor` | 시리얼 모니터 |
| `:McuPort` | 포트 선택 |
| `:McuFirmware` | STM32 `.bin` 경로 |
| `:McuClean` | PlatformIO clean |
| `:McuReset` | STM32 리셋 (`st-flash`) |

## 설정 파일

중앙 설정: `lua/autoforge-nvim/config.lua`

```lua
require("autoforge-nvim").setup({
  arduino_nvim = {
    board = "arduino:renesas_uno:unor4wifi",
    port = "/dev/ttyACM0",
    baudrate = 9600,
    use_default_keymaps = true,
  },
  autoforge_mcu = {
    arduino_backend = "arduino-nvim", -- builtin 으로 내장 arduino-cli 사용
    stm32_flash = "auto",
    flash_address = "0x08000000",
  },
})
```

## 사용 예

### Arduino

```bash
arduino-cli sketch new blink
cd blink
nvim blink.ino
# <leader>ab 보드 선택, <leader>ap 포트 선택
# <leader>ac 빌드, <leader>au 업로드, <leader>am 모니터
```

### STM32 (PlatformIO)

```bash
cd stm32-project
nvim src/main.c
# :McuBuildUpload
```

## 디렉터리 구조

```
nvim/
├── lazy-setup.lua              # lazy.nvim 원클릭 부트스트랩
├── lua/
│   ├── autoforge-nvim/         # .ino filetype + 통합 설정
│   ├── autoforge-mcu/          # STM32 / PlatformIO 백엔드
│   └── plugins/
│       ├── arduino-nvim.lua    # yuukiflow/Arduino-Nvim spec
│       ├── autoforge-mcu.lua   # 로컬 플러그인 spec
│       └── init.lua
└── plugin/autoforge-mcu.lua    # lazy 미사용 시 폴백 로드
```

## 라이선스

- `autoforge-mcu` — Apache-2.0 (AutoForge)
- [Arduino-Nvim](https://github.com/yuukiflow/Arduino-Nvim) — MIT

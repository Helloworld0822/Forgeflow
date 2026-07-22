local detect = require("autoforge-mcu.detect")
local runner = require("autoforge-mcu.runner")
local arduino = require("autoforge-mcu.arduino")
local stm32 = require("autoforge-mcu.stm32")
local platformio = require("autoforge-mcu.platformio")

local M = {}

M.config = {
  arduino_cli = "arduino-cli",
  platformio = "pio",
  st_flash = "st-flash",
  openocd = "openocd",
  baud = 115200,
  port = nil,
  fqbn = nil,
  pio_env = nil,
  firmware = nil,
  flash_address = "0x08000000",
  stm32_flash = "auto", -- auto | st-flash | openocd
  openocd_config = nil,
  arduino_backend = "arduino-nvim", -- arduino-nvim | builtin
  keymaps = true,
  map_prefix = "<leader>m",
}

local function arduino_nvim_ready()
  if M.config.arduino_backend ~= "arduino-nvim" then
    return false
  end
  local ok = pcall(require, "Arduino-Nvim")
  return ok
end

local function ino_cmd(cmd)
  if not arduino_nvim_ready() then
    return false
  end
  vim.cmd(cmd)
  return true
end

local function backend(project)
  if project.kind == "platformio" then
    return platformio, project.mcu
  end
  if project.kind == "arduino" then
    return arduino, "arduino"
  end
  if project.kind == "stm32" then
    return stm32, "stm32"
  end
  return nil, nil
end

function M.current_project()
  return detect.detect(vim.loop.cwd())
end

function M.info()
  local project = M.current_project()
  runner.notify_info(detect.describe(project) .. " @ " .. project.root)
end

function M.build()
  local project = M.current_project()
  if project.kind == "arduino" and ino_cmd("InoCheck") then
    return
  end
  local mod = backend(project)
  if not mod then
    runner.notify_error("지원하지 않는 프로젝트입니다.")
    return
  end
  mod.build(project, M.config)
end

function M.upload()
  local project = M.current_project()
  if project.kind == "arduino" and ino_cmd("InoUpload") then
    return
  end
  local mod = backend(project)
  if not mod then
    runner.notify_error("지원하지 않는 프로젝트입니다.")
    return
  end
  mod.upload(project, M.config)
end

function M.flash()
  M.upload()
end

function M.monitor()
  local project = M.current_project()
  if project.kind == "arduino" and ino_cmd("InoMonitor") then
    return
  end
  local mod = backend(project)
  if not mod then
    runner.notify_error("지원하지 않는 프로젝트입니다.")
    return
  end
  mod.monitor(project, M.config)
end

function M.clean()
  local project = M.current_project()
  if project.kind == "platformio" then
    platformio.clean(project, M.config)
    return
  end
  runner.notify_warn("이 프로젝트 타입은 clean을 지원하지 않습니다.")
end

function M.reset()
  local project = M.current_project()
  if project.mcu == "stm32" then
    stm32.reset(M.config)
    return
  end
  runner.notify_warn("Arduino 프로젝트는 reset을 지원하지 않습니다.")
end

local function pick_port(ports)
  if #ports == 0 then
    runner.notify_error("사용 가능한 포트가 없습니다.")
    return
  end

  local items = vim.tbl_map(function(p)
    return p.label or p.port
  end, ports)

  vim.ui.select(items, { prompt = "시리얼 포트 선택" }, function(choice)
    if not choice then
      return
    end
    for _, p in ipairs(ports) do
      if (p.label or p.port) == choice then
        M.config.port = p.port
        runner.notify_info("포트 설정: " .. p.port)
        return
      end
    end
  end)
end

function M.select_port()
  local project = M.current_project()
  if project.kind == "arduino" and ino_cmd("InoSelectPort") then
    return
  end
  local ports = {}
  if project.kind == "arduino" then
    ports = arduino.list_ports(M.config)
  elseif project.kind == "platformio" or project.mcu == "stm32" then
    ports = platformio.list_ports(M.config)
  else
    ports = arduino.list_ports(M.config)
    if #ports == 0 then
      ports = platformio.list_ports(M.config)
    end
  end
  pick_port(ports)
end

function M.set_fqbn()
  if ino_cmd("InoSelectBoard") then
    return
  end
  vim.ui.input({ prompt = "Arduino FQBN (예: arduino:avr:uno): ", default = M.config.fqbn or "" }, function(value)
    if value and value ~= "" then
      M.config.fqbn = value
      runner.notify_info("FQBN 설정: " .. value)
    end
  end)
end

function M.set_firmware()
  vim.ui.input({ prompt = "펌웨어 경로 (.bin): ", default = M.config.firmware or "" }, function(value)
    if value and value ~= "" then
      M.config.firmware = value
      runner.notify_info("펌웨어 경로: " .. value)
    end
  end)
end

function M.build_upload()
  local project = M.current_project()
  if project.kind == "arduino" then
    if ino_cmd("InoCheck") then
      vim.defer_fn(function()
        ino_cmd("InoUpload")
      end, 500)
    end
    return
  end
  local mod = backend(project)
  if not mod then
    runner.notify_error("지원하지 않는 프로젝트입니다.")
    return
  end
  mod.build(project, M.config, {
    on_exit = function(code)
      if code == 0 then
        mod.upload(project, M.config)
      else
        runner.notify_error("빌드 실패로 업로드를 건너뜁니다.")
      end
    end,
  })
end

local function register_commands()
  local cmds = {
    { "McuInfo", M.info, "MCU 프로젝트 정보" },
    { "McuBuild", M.build, "빌드" },
    { "McuUpload", M.upload, "업로드/플래시" },
    { "McuFlash", M.flash, "STM32 플래시 (McuUpload와 동일)" },
    { "McuMonitor", M.monitor, "시리얼 모니터" },
    { "McuClean", M.clean, "빌드 산출물 정리 (PlatformIO)" },
    { "McuReset", M.reset, "STM32 리셋 (st-flash)" },
    { "McuPort", M.select_port, "시리얼 포트 선택" },
    { "McuFqbn", M.set_fqbn, "Arduino FQBN 설정" },
    { "McuFirmware", M.set_firmware, "STM32 펌웨어 경로 설정" },
    { "McuBuildUpload", M.build_upload, "빌드 후 업로드" },
  }

  for _, def in ipairs(cmds) do
    vim.api.nvim_create_user_command(def[1], def[2], { desc = def[3] })
  end
end

local function register_keymaps()
  if not M.config.keymaps then
    return
  end

  local prefix = M.config.map_prefix or "<leader>m"
  local maps = {
    { prefix .. "i", M.info, "MCU info" },
    { prefix .. "b", M.build, "MCU build" },
    { prefix .. "u", M.upload, "MCU upload" },
    { prefix .. "m", M.monitor, "MCU monitor" },
    { prefix .. "p", M.select_port, "MCU port" },
    { prefix .. "B", M.build_upload, "MCU build+upload" },
  }

  for _, map in ipairs(maps) do
    vim.keymap.set("n", map[1], map[2], { desc = map[3], silent = true })
  end
end

function M.setup(opts)
  M.config = vim.tbl_deep_extend("force", M.config, opts or {})
  register_commands()
  register_keymaps()
end

return M

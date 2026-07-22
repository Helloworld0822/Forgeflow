local runner = require("autoforge-mcu.runner")
local platformio = require("autoforge-mcu.platformio")

local M = {}

local function st_flash(config)
  return runner.tool_path("st-flash", "st_flash", config)
end

local function openocd(config)
  return runner.tool_path("openocd", "openocd", config)
end

local function find_firmware(root, config)
  if config.firmware and config.firmware ~= "" then
    return config.firmware
  end

  local patterns = {
    "build/*.bin",
    "build/*.elf",
    "Debug/*.bin",
    "Debug/*.elf",
    "cmake-build-*/**/*.bin",
    "**/firmware.bin",
  }

  for _, pattern in ipairs(patterns) do
    local matches = vim.fn.glob(root .. "/" .. pattern, false, true)
    if #matches > 0 then
      for _, path in ipairs(matches) do
        if path:match("%.bin$") then
          return path
        end
      end
      return matches[1]
    end
  end

  return nil
end

function M.list_ports(config)
  return platformio.list_ports(config)
end

function M.build(project, config, opts)
  if vim.fn.executable("pio") == 1 or vim.fn.executable(config.platformio or "pio") == 1 then
    return platformio.build(project, config, opts)
  end

  local makefile = project.root .. "/Makefile"
  if vim.loop.fs_stat(makefile) then
    runner.run_job({ "make" }, {
      cwd = project.root,
      title = "make",
      on_exit = opts and opts.on_exit,
    })
    return
  end

  local cmake = project.root .. "/CMakeLists.txt"
  if vim.loop.fs_stat(cmake) then
    local build_dir = project.root .. "/build"
    vim.fn.mkdir(build_dir, "p")
    runner.run_job({ "cmake", "..", "-DCMAKE_BUILD_TYPE=Debug" }, {
      cwd = build_dir,
      title = "cmake configure",
      on_exit = function(code)
        if code ~= 0 then
          return
        end
        runner.run_job({ "cmake", "--build", "." }, {
          cwd = build_dir,
          title = "cmake build",
          on_exit = opts and opts.on_exit,
        })
      end,
    })
    return
  end

  runner.notify_error("STM32 빌드 도구를 찾을 수 없습니다 (pio, make, cmake).")
end

function M.flash_stlink(project, config, opts)
  opts = opts or {}
  local bin = st_flash(config)
  if not runner.ensure_tool(bin, "st-flash") then
    return
  end

  local firmware = find_firmware(project.root, config)
  if not firmware then
    runner.notify_error("펌웨어 .bin 파일을 찾을 수 없습니다. 먼저 빌드하거나 :McuFirmware 로 경로를 지정하세요.")
    return
  end

  local address = config.flash_address or "0x08000000"
  local cmd = { bin, "write", firmware, address }
  runner.run_job(cmd, {
    cwd = project.root,
    title = "st-flash write",
    on_exit = opts.on_exit,
  })
end

function M.flash_openocd(project, config, opts)
  opts = opts or {}
  local ocd = openocd(config)
  if not runner.ensure_tool(ocd, "openocd") then
    return
  end

  local cfg = config.openocd_config
  if not cfg or cfg == "" then
    local candidates = {
      project.root .. "/openocd.cfg",
      project.root .. "/interface/stlink.cfg",
    }
    for _, path in ipairs(candidates) do
      if vim.loop.fs_stat(path) then
        cfg = path
        break
      end
    end
  end

  local firmware = find_firmware(project.root, config)
  local cmd
  if cfg then
    cmd = { ocd, "-f", cfg }
    if firmware and not cfg:match("program") then
      table.insert(cmd, "-c")
      table.insert(cmd, string.format("program %s verify reset exit", firmware))
    end
  else
    if not firmware then
      runner.notify_error("openocd 설정 또는 펌웨어 파일이 필요합니다.")
      return
    end
    cmd = {
      ocd,
      "-f", "interface/stlink.cfg",
      "-f", "target/stm32f4x.cfg",
      "-c", string.format("program %s verify reset exit", firmware),
    }
  end

  runner.run_job(cmd, {
    cwd = project.root,
    title = "openocd flash",
    on_exit = opts.on_exit,
  })
end

function M.upload(project, config, opts)
  if vim.fn.executable("pio") == 1 or vim.fn.executable(config.platformio or "pio") == 1 then
    return platformio.upload(project, config, opts)
  end

  local method = config.stm32_flash or "auto"
  if method == "openocd" then
    return M.flash_openocd(project, config, opts)
  end
  if method == "st-flash" then
    return M.flash_stlink(project, config, opts)
  end

  if runner.executable(st_flash(config)) then
    return M.flash_stlink(project, config, opts)
  end
  if runner.executable(openocd(config)) then
    return M.flash_openocd(project, config, opts)
  end

  runner.notify_error("STM32 업로드 도구를 찾을 수 없습니다 (pio, st-flash, openocd).")
end

function M.monitor(project, config)
  if vim.fn.executable("pio") == 1 or vim.fn.executable(config.platformio or "pio") == 1 then
    return platformio.monitor(project, config)
  end

  local port = config.port
  if not port or port == "" then
    local ports = M.list_ports(config)
    if #ports == 0 then
      runner.notify_error("시리얼 포트가 없습니다. :McuPort 로 포트를 지정하세요.")
      return
    end
    port = ports[1].port
  end

  local baud = tostring(config.baud or 115200)
  if runner.executable("picocom") then
    runner.open_terminal({ "picocom", port, "-b", baud }, {
      cwd = project.root,
      title = "stm32-serial",
    })
    return
  end

  if runner.executable("minicom") then
    runner.open_terminal({ "minicom", "-D", port, "-b", baud }, {
      cwd = project.root,
      title = "stm32-serial",
    })
    return
  end

  runner.notify_error("시리얼 모니터 도구가 없습니다 (pio, picocom, minicom).")
end

function M.reset(config)
  local bin = st_flash(config)
  if runner.executable(bin) then
    runner.run_job({ bin, "reset" }, { title = "st-flash reset" })
    return
  end
  runner.notify_error("st-flash가 없어 리셋할 수 없습니다.")
end

return M

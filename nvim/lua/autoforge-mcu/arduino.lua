local runner = require("autoforge-mcu.runner")

local M = {}

local function arduino_cli(config)
  return runner.tool_path("arduino-cli", "arduino_cli", config)
end

function M.list_ports(config)
  local cli = arduino_cli(config)
  if not runner.ensure_tool(cli, "arduino-cli") then
    return {}
  end

  local out, err = runner.run_capture({ cli, "board", "list", "--format", "json" })
  if not out then
    runner.notify_error(err or "포트 목록 조회 실패")
    return {}
  end

  local ok, data = pcall(vim.json.decode, out)
  if not ok or type(data) ~= "table" then
    return {}
  end

  local ports = {}
  for _, entry in ipairs(data) do
    local port = entry.port or entry
    local address = port.address or port.Address
    if address and address ~= "" then
      table.insert(ports, {
        port = address,
        label = string.format("%s (%s)", address, port.protocol or "serial"),
        fqbn = entry.matching_boards and entry.matching_boards[1] and entry.matching_boards[1].fqbn,
      })
    end
  end
  return ports
end

function M.resolve_fqbn(project, config)
  if config.fqbn and config.fqbn ~= "" then
    return config.fqbn
  end

  local sketch_yaml = project.root .. "/sketch.yaml"
  local f = io.open(sketch_yaml, "r")
  if f then
    local content = f:read("*a")
    f:close()
    local fqbn = content:match("fqbn%s*:%s*([%w:%%-%.]+)")
    if fqbn then
      return fqbn
    end
  end

  local ports = M.list_ports(config)
  for _, p in ipairs(ports) do
    if p.fqbn then
      return p.fqbn
    end
  end

  return nil
end

function M.build(project, config, opts)
  opts = opts or {}
  local cli = arduino_cli(config)
  if not runner.ensure_tool(cli, "arduino-cli") then
    return
  end

  local fqbn = M.resolve_fqbn(project, config)
  if not fqbn then
    runner.notify_error("FQBN이 없습니다. :McuFqbn 으로 보드 FQBN을 설정하세요.")
    return
  end

  local sketch = project.marker
  if not sketch:match("%.ino$") then
    local matches = vim.fn.glob(project.root .. "/*.ino", false, true)
    sketch = matches[1]
  end
  if not sketch then
    runner.notify_error("*.ino 스케치를 찾을 수 없습니다.")
    return
  end

  local cmd = { cli, "compile", "-b", fqbn, sketch }
  runner.run_job(cmd, {
    cwd = project.root,
    title = "arduino-cli compile",
    on_exit = opts.on_exit,
  })
end

function M.upload(project, config, opts)
  opts = opts or {}
  local cli = arduino_cli(config)
  if not runner.ensure_tool(cli, "arduino-cli") then
    return
  end

  local fqbn = M.resolve_fqbn(project, config)
  if not fqbn then
    runner.notify_error("FQBN이 없습니다.")
    return
  end

  local port = config.port
  if not port or port == "" then
    local ports = M.list_ports(config)
    if #ports == 0 then
      runner.notify_error("연결된 시리얼 포트가 없습니다.")
      return
    end
    port = ports[1].port
  end

  local sketch = project.marker
  if not sketch:match("%.ino$") then
    sketch = vim.fn.glob(project.root .. "/*.ino", false, true)[1]
  end

  local cmd = { cli, "upload", "-b", fqbn, "-p", port, sketch }
  runner.run_job(cmd, {
    cwd = project.root,
    title = "arduino-cli upload",
    on_exit = opts.on_exit,
  })
end

function M.monitor(project, config)
  local cli = arduino_cli(config)
  if not runner.ensure_tool(cli, "arduino-cli") then
    return
  end

  local port = config.port
  if not port or port == "" then
    local ports = M.list_ports(config)
    if #ports == 0 then
      runner.notify_error("연결된 시리얼 포트가 없습니다.")
      return
    end
    port = ports[1].port
  end

  local baud = tostring(config.baud or 115200)
  runner.open_terminal({ cli, "monitor", "-p", port, "-c", "baudrate=" .. baud }, {
    cwd = project.root,
    title = "arduino-monitor",
  })
end

return M

local runner = require("autoforge-mcu.runner")

local M = {}

local function pio(config)
  return runner.tool_path("pio", "platformio", config)
end

function M.list_ports(config)
  local bin = pio(config)
  if not runner.ensure_tool(bin, "platformio (pio)") then
    return {}
  end

  local out, err = runner.run_capture({ bin, "device", "list", "--serial", "--json-output" })
  if not out then
    runner.notify_error(err or "PIO 포트 목록 조회 실패")
    return {}
  end

  local ok, data = pcall(vim.json.decode, out)
  if not ok or type(data) ~= "table" then
    return {}
  end

  local ports = {}
  for _, entry in ipairs(data) do
    if entry.port and entry.port ~= "" then
      table.insert(ports, {
        port = entry.port,
        label = string.format("%s (%s)", entry.port, entry.description or "serial"),
      })
    end
  end
  return ports
end

function M.build(project, config, opts)
  opts = opts or {}
  local bin = pio(config)
  if not runner.ensure_tool(bin, "platformio (pio)") then
    return
  end

  local cmd = { bin, "run" }
  if config.pio_env and config.pio_env ~= "" then
    table.insert(cmd, "-e")
    table.insert(cmd, config.pio_env)
  end

  runner.run_job(cmd, {
    cwd = project.root,
    title = "pio run",
    on_exit = opts.on_exit,
  })
end

function M.upload(project, config, opts)
  opts = opts or {}
  local bin = pio(config)
  if not runner.ensure_tool(bin, "platformio (pio)") then
    return
  end

  local cmd = { bin, "run", "-t", "upload" }
  if config.pio_env and config.pio_env ~= "" then
    table.insert(cmd, "-e")
    table.insert(cmd, config.pio_env)
  end
  if config.port and config.port ~= "" then
    table.insert(cmd, "--upload-port")
    table.insert(cmd, config.port)
  end

  runner.run_job(cmd, {
    cwd = project.root,
    title = "pio upload",
    on_exit = opts.on_exit,
  })
end

function M.monitor(project, config)
  local bin = pio(config)
  if not runner.ensure_tool(bin, "platformio (pio)") then
    return
  end

  local cmd = { bin, "device", "monitor" }
  if config.pio_env and config.pio_env ~= "" then
    table.insert(cmd, "-e")
    table.insert(cmd, config.pio_env)
  end
  if config.port and config.port ~= "" then
    table.insert(cmd, "--port")
    table.insert(cmd, config.port)
  end
  if config.baud then
    table.insert(cmd, "--baud")
    table.insert(cmd, tostring(config.baud))
  end

  runner.open_terminal(cmd, {
    cwd = project.root,
    title = "pio-monitor",
  })
end

function M.clean(project, config)
  local bin = pio(config)
  if not runner.ensure_tool(bin, "platformio (pio)") then
    return
  end

  local cmd = { bin, "run", "-t", "clean" }
  if config.pio_env and config.pio_env ~= "" then
    table.insert(cmd, "-e")
    table.insert(cmd, config.pio_env)
  end

  runner.run_job(cmd, {
    cwd = project.root,
    title = "pio clean",
  })
end

return M

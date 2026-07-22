local M = {}

local function file_exists(path)
  return vim.loop.fs_stat(path) ~= nil
end

local function find_upward(names, start_dir)
  local dir = start_dir or vim.loop.cwd()
  while dir and dir ~= "/" do
    for _, name in ipairs(names) do
      local path = dir .. "/" .. name
      if file_exists(path) then
        return path, dir
      end
    end
    dir = vim.fn.fnamemodify(dir, ":h")
  end
  return nil, start_dir or vim.loop.cwd()
end

local function glob_one(pattern, root)
  local matches = vim.fn.glob(root .. "/" .. pattern, false, true)
  if #matches > 0 then
    return matches[1]
  end
  return nil
end

local function read_file(path)
  local f = io.open(path, "r")
  if not f then
    return nil
  end
  local content = f:read("*a")
  f:close()
  return content
end

function M.detect(root)
  root = root or vim.loop.cwd()

  local pio_ini = find_upward({ "platformio.ini" }, root)
  if pio_ini then
    local content = read_file(pio_ini) or ""
    local platform = content:match("platform%s*=%s*([%w%-_]+)") or ""
    local framework = content:match("framework%s*=%s*([%w%-_]+)") or ""
    local mcu = "platformio"
    if platform:find("ststm32", 1, true) or framework:find("stm32", 1, true) then
      mcu = "stm32"
    elseif platform:find("atmelavr", 1, true)
      or platform:find("espressif", 1, true)
      or framework:find("arduino", 1, true)
    then
      mcu = "arduino"
    end
    return {
      kind = "platformio",
      mcu = mcu,
      root = select(2, find_upward({ "platformio.ini" }, root)),
      marker = pio_ini,
      platform = platform,
      framework = framework,
    }
  end

  local sketch = glob_one("*.ino", root) or find_upward({ "sketch.yaml" }, root)
  if sketch then
    local sketch_root = vim.fn.fnamemodify(sketch, ":h")
    return {
      kind = "arduino",
      mcu = "arduino",
      root = sketch_root,
      marker = sketch,
    }
  end

  local ioc = glob_one("*.ioc", root)
  local stm32_ld = glob_one("STM32*.ld", root) or glob_one("*.ld", root)
  local cmake = find_upward({ "CMakeLists.txt" }, root)
  if ioc or (cmake and (glob_one("Drivers/STM32*", root) or stm32_ld)) then
    return {
      kind = "stm32",
      mcu = "stm32",
      root = root,
      marker = ioc or cmake or stm32_ld,
      ioc = ioc,
    }
  end

  local makefile = find_upward({ "Makefile" }, root)
  if makefile and read_file(makefile) and read_file(makefile):find("arm-none-eabi", 1, true) then
    return {
      kind = "stm32",
      mcu = "stm32",
      root = select(2, find_upward({ "Makefile" }, root)),
      marker = makefile,
    }
  end

  return {
    kind = "unknown",
    mcu = "unknown",
    root = root,
    marker = nil,
  }
end

function M.describe(project)
  if project.kind == "unknown" then
    return "알 수 없는 프로젝트 (platformio.ini, *.ino, STM32 *.ioc 필요)"
  end
  local marker = project.marker and vim.fn.fnamemodify(project.marker, ":t") or "?"
  return string.format("%s / %s (%s)", project.mcu:upper(), project.kind, marker)
end

return M

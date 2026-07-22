local M = {}

local function notify(level, msg)
  vim.notify(msg, level, { title = "autoforge-mcu" })
end

function M.notify_info(msg)
  notify(vim.log.levels.INFO, msg)
end

function M.notify_warn(msg)
  notify(vim.log.levels.WARN, msg)
end

function M.notify_error(msg)
  notify(vim.log.levels.ERROR, msg)
end

function M.tool_path(name, config_key, config)
  return (config and config[config_key]) or name
end

function M.executable(path)
  return vim.fn.executable(path) == 1
end

function M.ensure_tool(path, label)
  if not M.executable(path) then
    M.notify_error(string.format("%s을(를) 찾을 수 없습니다: %s", label, path))
    return false
  end
  return true
end

function M.run_job(cmd, opts)
  opts = opts or {}
  local title = opts.title or table.concat(cmd, " ")
  local cwd = opts.cwd or vim.loop.cwd()

  M.notify_info(string.format("실행: %s", title))

  local lines = {}
  local job_id = vim.fn.jobstart(cmd, {
    cwd = cwd,
    stdout_buffered = true,
    stderr_buffered = true,
    on_stdout = function(_, data)
      if data then
        vim.list_extend(lines, data)
      end
    end,
    on_stderr = function(_, data)
      if data then
        vim.list_extend(lines, data)
      end
    end,
    on_exit = function(_, code)
      if opts.on_exit then
        opts.on_exit(code, lines)
        return
      end
      if code == 0 then
        M.notify_info(string.format("완료: %s", title))
      else
        local tail = vim.trim(table.concat(lines, "\n"))
        if #tail > 400 then
          tail = tail:sub(-400)
        end
        M.notify_error(string.format("실패 (%d): %s\n%s", code, title, tail))
      end
    end,
  })

  if job_id <= 0 then
    M.notify_error(string.format("작업 시작 실패: %s", title))
    return nil
  end

  return job_id
end

function M.run_capture(cmd, cwd)
  local result = vim.system(cmd, { cwd = cwd, text = true })
  if result.code ~= 0 then
    return nil, result.stderr ~= "" and result.stderr or result.stdout
  end
  return result.stdout, nil
end

function M.open_terminal(cmd, opts)
  opts = opts or {}
  local cwd = opts.cwd or vim.loop.cwd()
  local title = opts.title or "MCU"

  if vim.fn.exists(":ToggleTerm") == 2 then
    vim.cmd(string.format("ToggleTerm direction=horizontal cmd=%s", vim.fn.escape(table.concat(cmd, " "), " ")))
    return
  end

  vim.cmd("botright split")
  vim.cmd("terminal")
  local buf = vim.api.nvim_get_current_buf()
  vim.api.nvim_buf_set_name(buf, "autoforge-mcu://" .. title)
  vim.fn.chdir(cwd)
  vim.cmd("startinsert")
  vim.fn.termopen(cmd, { cwd = cwd })
end

return M

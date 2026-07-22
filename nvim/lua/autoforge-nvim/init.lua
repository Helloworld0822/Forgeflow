local config = require("autoforge-nvim.config")

local M = {}

function M.setup(opts)
  if opts then
    config.arduino_nvim = vim.tbl_deep_extend("force", config.arduino_nvim, opts.arduino_nvim or {})
    config.autoforge_mcu = vim.tbl_deep_extend("force", config.autoforge_mcu, opts.autoforge_mcu or {})
  end

  vim.filetype.add({
    extension = {
      ino = "arduino",
    },
  })
end

function M.lazy_specs()
  return require("plugins.init")
end

return M

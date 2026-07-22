local config = require("autoforge-nvim.config")

local nvim_root = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":h:h:h")

return {
  dir = nvim_root,
  name = "autoforge-mcu",
  config = function()
    vim.g.autoforge_nvim_lazy = true
    require("autoforge-mcu").setup(vim.deepcopy(config.autoforge_mcu))
  end,
}

local config = require("autoforge-nvim.config")

return {
  "yuukiflow/Arduino-Nvim",
  ft = "arduino",
  dependencies = {
    "nvim-telescope/telescope.nvim",
    "neovim/nvim-lspconfig",
  },
  opts = vim.deepcopy(config.arduino_nvim),
  config = function(_, opts)
    require("Arduino-Nvim").setup(opts)
  end,
}

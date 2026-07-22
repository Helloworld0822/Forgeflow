--- AutoForge nvim + yuukiflow/Arduino-Nvim lazy.nvim 부트스트랩 예시
---
--- ~/.config/nvim/init.lua 에 추가:
---
---   local autoforge_nvim = vim.fn.expand("~/code/AutoForge/nvim")
---   vim.opt.rtp:prepend(autoforge_nvim)
---   require("autoforge-nvim").setup({})
---   require("lazy").setup(require("plugins.init"), {
---     root = vim.fn.stdpath("data") .. "/lazy/autoforge",
---   })

local autoforge_nvim = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":h")
vim.opt.rtp:prepend(autoforge_nvim)

require("autoforge-nvim").setup({})

local lazypath = vim.fn.stdpath("data") .. "/lazy/lazy.nvim"
if not vim.loop.fs_stat(lazypath) then
  vim.fn.system({
    "git",
    "clone",
    "--filter=blob:none",
    "https://github.com/folke/lazy.nvim.git",
    "--branch=stable",
    lazypath,
  })
end
vim.opt.rtp:prepend(lazypath)

require("lazy").setup(require("plugins.init"), {
  root = vim.fn.stdpath("data") .. "/lazy/autoforge",
  install = { colorscheme = {} },
  checker = { enabled = false },
})

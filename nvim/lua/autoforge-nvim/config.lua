---@class AutoforgeNvimConfig
local M = {
  ---@type table Arduino-Nvim opts (yuukiflow/Arduino-Nvim)
  arduino_nvim = {
    config_file = ".arduino_config.lua",
    board = "arduino:avr:uno",
    port = "/dev/ttyACM0",
    baudrate = 115200,
    picker_backend = "telescope",
    use_default_keymaps = true,
    use_default_commands = true,
    keymaps = {},
    compile_options = {},
  },
  ---@type table autoforge-mcu opts (STM32 / PlatformIO)
  autoforge_mcu = {
    arduino_backend = "arduino-nvim", -- arduino-nvim | builtin
    keymaps = true,
    map_prefix = "<leader>m",
    stm32_flash = "auto",
    baud = 115200,
  },
}

return M

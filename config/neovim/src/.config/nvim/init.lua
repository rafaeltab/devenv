package.path = package.path .. ";" .. vim.fn.stdpath("config") .. "/lua/?.lua"


-- The nvim configuration for nvim
require 'base'

require 'keybinds'
require 'filetypes'
require 'plugins.appearance'
require 'plugins.completion'
require 'plugins.file_exploring'
require 'plugins.formatting'
require 'plugins.git'
require 'plugins.harpoon'
require 'plugins.notetaking'
require 'plugins.telescope'
require 'plugins.testing'
require 'plugins.treesitter'
require 'plugins.utilities'
require 'plugins.ai'

require 'languages.dart'
-- require 'languages.java'
require 'languages.json'
require 'languages.markdown'
require 'languages.terraform'
require 'languages.typescript'
require 'languages.yaml'
-- require 'languages.csharp'
require 'languages.powershell'
require 'languages.bicep'
require 'languages.swift'
require 'languages.go'
require 'languages.powershell'
require 'languages.bicep'
require 'languages.rust'
require 'languages.nix'

require 'lsp'

-- vim.opt.rocks.hererocks = false
vim.opt.termguicolors = true
Plugins:add({
  {
    'declancm/cinnamon.nvim',
    config = function()
      require('cinnamon').setup({
        disabled = false,
        keymaps = {
          basic = true,
          extra = true,
        },
        options = {
          delay = 1,
          mode = "window",
          max_delta = {
            line = 2048,
            column = 2048,
          }
        }
      })
    end
  },
  {
    'navarasu/onedark.nvim',
    priority = 999,
    opts = {
      style = "darker",
      transparent = true,
    },
  },
  {
    "vhyrro/luarocks.nvim",
    priority = 1000,     -- Very high priority is required, luarocks.nvim should run as the first plugin in your config.
    config = true,
  }
})
Setup()


vim.cmd.colorscheme 'onedark'

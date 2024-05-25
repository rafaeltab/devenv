require 'keybinds'
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

require 'lsp';

require "utils.on_attach"
require "utils.on_load"
require "utils.plugins"

local languages = require 'languages.languages'

Map = function(mode, lhs, rhs, opts)
  local options = { noremap = true, silent = true }
  if opts then
    options = vim.tbl_extend("force", options, opts)
  end
  vim.keymap.set(mode, lhs, rhs, options)
end

function setup(plugins)
  -- The base configuration for `nvim` and `vscode-nvim`
  -- Set <space> as the leader key
  -- See `:help mapleader`
  --  NOTE: Must happen before plugins are required (otherwise wrong leader will be used)
  vim.g.mapleader = ' '
  vim.g.maplocalleader = ' '
  vim.g.netrw_liststyle = 3

  vim.filetype.add({
    extension = {
      mdx = "mdx"
    }
  })

  -- Install package manager
  --    https://github.com/folke/lazy.nvim
  --    `:help lazy.nvim.txt` for more info
  local lazypath = vim.fn.stdpath 'data' .. '/lazy/lazy.nvim'
  if not vim.loop.fs_stat(lazypath) then
    vim.fn.system {
      'git',
      'clone',
      '--filter=blob:none',
      'https://github.com/folke/lazy.nvim.git',
      '--branch=stable', -- latest stable release
      lazypath,
    }
  end
  vim.opt.rtp:prepend(lazypath)

  -- NOTE: Here is where you install your plugins.
  --  You can configure plugins using the `config` key.
  --
  --  You can also configure plugins after the setup call,
  --    as they will be available in your neovim runtime.
  require('lazy').setup({
    plugins,
    languages.plugins,
    Plugins:get_plugins()
  }, {})

  OnLoad:load()

  require 'options'
  -- The line beneath this is called `modeline`. See `:help modeline`
end

require "utils.on_attach"
require "utils.on_load"
require "utils.plugins"
require "utils.languages_v2"
require "utils.utils"
require "utils.types"

-- Fix later
function Setup()
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
      '--branch=stable',       -- latest stable release
      lazypath,
    }
  end
  vim.opt.rtp:prepend(lazypath)

  require('lazy').setup({
    Plugins:get_plugins()
  }, {})

  OnLoad:load()

  require 'options'
end

require "utils.on_attach"
require "utils.on_load"
require "utils.plugins"
require "utils.languages"
require "utils.utils"

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
require 'filetypes'

require 'languages.dart'
require 'languages.java'
require 'languages.json'
require 'languages.markdown'
require 'languages.terraform'
require 'languages.typescript'
require 'languages.yaml'
require 'languages.csharp'

require 'lsp';

function setup()
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

    require('lazy').setup({
        Plugins:get_plugins()
    }, {})

    OnLoad:load()

    require 'options'
end

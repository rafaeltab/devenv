require 'plugins.completion'
local languages = require 'languages.languages'

local highlight = {
  "RainbowRed",
  "RainbowYellow",
  "RainbowBlue",
  "RainbowOrange",
  "RainbowGreen",
  "RainbowViolet",
  "RainbowCyan",
}

local map = function(mode, lhs, rhs, opts)
  local options = { noremap = true, silent = true }
  if opts then
    options = vim.tbl_extend("force", options, opts)
  end
  vim.keymap.set(mode, lhs, rhs, options)
end

function setup(plugins)
  -- The base configuration for nvim and vscode-nvim
  -- Set <space> as the leader key
  -- See `:help mapleader`
  --  NOTE: Must happen before plugins are required (otherwise wrong leader will be used)
  vim.g.mapleader = ' '
  vim.g.maplocalleader = ' '
  vim.g.netrw_liststyle = 3

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
    -- NOTE: First, some plugins that don't require any configuration

    -- Git related plugins
    'tpope/vim-fugitive',
    'tpope/vim-rhubarb',
    {
      'stevearc/oil.nvim',
      opts = {
        view_options = {
          show_hidden = true,
        }
      },
      -- Optional dependencies
      dependencies = { "nvim-tree/nvim-web-devicons" },
    },
    {
      "epwalsh/obsidian.nvim",
      version = "*", -- recommended, use latest release instead of latest commit
      lazy = true,
      ft = "markdown",
      -- Replace the above line with this if you only want to load obsidian.nvim for markdown files in your vault:
      -- event = {
      --   -- If you want to use the home shortcut '~' here you need to call 'vim.fn.expand'.
      --   -- E.g. "BufReadPre " .. vim.fn.expand "~" .. "/my-vault/**.md"
      --   "BufReadPre path/to/my-vault/**.md",
      --   "BufNewFile path/to/my-vault/**.md",
      -- },
      dependencies = {
        -- Required.
        "nvim-lua/plenary.nvim",

        -- see below for full list of optional dependencies 👇
      },
      opts = {
        workspaces = {
          {
            name = "Coding knowledge",
            path = "/home/rafaeltab/home/notes/coding_knowledge",
          },
          {
            name = "rafaeltab command line tool",
            path = "/home/rafaeltab/home/notes/rafaeltab command line tool",
          },
        },
      },
    },
    {
      "nvim-neotest/neotest",
      dependencies = {
        "nvim-lua/plenary.nvim",
        "nvim-treesitter/nvim-treesitter",
        "antoinemadec/FixCursorHold.nvim",
        'sidlatau/neotest-dart',
        'rouge8/neotest-rust'
      },
      config = function()
        require("neotest").setup({
          adapters = {
            require("neotest-dart") {
              command = "flutter",
              use_lsp = true,
              custom_test_method_names = { "blocTest" }
            },
            require("neotest-rust")
          },
          consumers = { require("neotest").diagnostic, require("neotest").status }
        })
      end
    },
    {
      -- Adds git releated signs to the gutter, as well as utilities for managing changes
      'lewis6991/gitsigns.nvim',
      opts = {
        -- See `:help gitsigns.txt`
        signs = {
          add = { text = '+' },
          change = { text = '~' },
          delete = { text = '_' },
          topdelete = { text = '‾' },
          changedelete = { text = '~' },
        },
      },
    },

    'tpope/vim-sleuth',
    -- Detect tabstop and shiftwidth automatically,
    "ThePrimeagen/vim-be-good",
    --  The configuration is done below. Search for lspconfig to find it below.
    {
      -- LSP Configuration & Plugins
      'neovim/nvim-lspconfig',
      dependencies = {
        -- Automatically install LSPs to stdpath for neovim
        'williamboman/mason.nvim',
        'williamboman/mason-lspconfig.nvim',

        -- Useful status updates for LSP
        -- NOTE: `opts = {}` is the same as calling `require('fidget').setup({})`
        {
          'j-hui/fidget.nvim',
          opts = {},
          tag = "legacy",
        },

        -- Additional lua configuration, makes nvim stuff amazing!
        'folke/neodev.nvim',
      },
    },

    -- Useful plugin to show you pending keybinds.,
    { 'folke/which-key.nvim',          opts = {} },

    {
      -- Set lualine as statusline
      'nvim-lualine/lualine.nvim',
      -- See `:help lualine.txt`
      opts = {
        options = {
          icons_enabled = true,
          theme = 'horizon',
          component_separators = { left = '', right = '' },
          section_separators = { left = '', right = '' },
        },
      },
    },
    {
      'theprimeagen/harpoon'
    },
    {
      -- Add indentation guides even on blank lines
      'lukas-reineke/indent-blankline.nvim',
      main = "ibl",
      -- Enable `lukas-reineke/indent-blankline.nvim`
      -- See `:help indent_blankline.txt`
    },

    -- "gc" to comment visual regions/lines
    { 'numToStr/Comment.nvim',         opts = {} },

    -- Fuzzy Finder (files, lsp, etc)
    { 'nvim-telescope/telescope.nvim', version = '*', dependencies = { 'nvim-lua/plenary.nvim' } },

    -- Fuzzy Finder Algorithm which requires local dependencies to be built.
    -- Only load if `make` is available. Make sure you have the system
    -- requirements installed.
    {
      'nvim-telescope/telescope-fzf-native.nvim',
      -- NOTE: If you are having trouble with this installation,
      --       refer to the README for telescope-fzf-native for more instructions.
      build = 'make',
      cond = function()
        return vim.fn.executable 'make' == 1
      end,
    },

    {
      -- Highlight, edit, and navigate code
      'nvim-treesitter/nvim-treesitter',
      dependencies = {
        'nvim-treesitter/nvim-treesitter-textobjects',
      },
      config = function()
        pcall(require('nvim-treesitter.install').update { with_sync = true })
      end,
    },
    plugins,
    completion.plugins(completion),
    languages.plugins
  }, {})


  local hooks = require "ibl.hooks"
  -- create the highlight groups in the highlight setup hook, so they are reset
  -- every time the colorscheme changes
  hooks.register(hooks.type.HIGHLIGHT_SETUP, function()
    vim.api.nvim_set_hl(0, "RainbowRed", { fg = "#E06C75" })
    vim.api.nvim_set_hl(0, "RainbowYellow", { fg = "#E5C07B" })
    vim.api.nvim_set_hl(0, "RainbowBlue", { fg = "#61AFEF" })
    vim.api.nvim_set_hl(0, "RainbowOrange", { fg = "#D19A66" })
    vim.api.nvim_set_hl(0, "RainbowGreen", { fg = "#98C379" })
    vim.api.nvim_set_hl(0, "RainbowViolet", { fg = "#C678DD" })
    vim.api.nvim_set_hl(0, "RainbowCyan", { fg = "#56B6C2" })
  end)
  vim.g.rainbow_delimiters = { highlight = highlight }
  require("ibl").setup {
    indent = {
      char = '┊',
    },
    scope = {
      highlight = highlight
    }
  }
  hooks.register(hooks.type.SCOPE_HIGHLIGHT, hooks.builtin.scope_highlight_from_extmark)

  -- [[ Basic Keymaps ]]

  -- Keymaps for better default experience
  -- See `:help vim.keymap.set()`
  vim.keymap.set({ 'n', 'v' }, '<Space>', '<Nop>', { silent = true })

  -- vim.keymap.set('n', '<leader>pv', vim.cmd.Ex)
  map("n", "<leader>pv", function()
    local oil = require("oil")
    oil.open(oil.get_current_dir())
  end, { desc = "Open Oil file manager in directory of current buffer" })

  -- Remap for dealing with word wrap
  vim.keymap.set('n', 'k', "v:count == 0 ? 'gk' : 'k'", { expr = true, silent = true })
  vim.keymap.set('n', 'j', "v:count == 0 ? 'gj' : 'j'", { expr = true, silent = true })

  -- [[ Highlight on yank ]]
  -- See `:help vim.highlight.on_yank()`
  local highlight_group = vim.api.nvim_create_augroup('YankHighlight', { clear = true })
  vim.api.nvim_create_autocmd('TextYankPost', {
    callback = function()
      vim.highlight.on_yank()
    end,
    group = highlight_group,
    pattern = '*',
  })

  require 'plugins.telescope';

  local harpoon_mark = require("harpoon.mark")
  local harpoon_ui = require("harpoon.ui")

  vim.keymap.set("n", "<leader>ha", harpoon_mark.add_file)
  vim.keymap.set("n", "<C-e>", harpoon_ui.toggle_quick_menu)

  vim.keymap.set("n", "<C-h>", function() harpoon_ui.nav_file(1) end)
  vim.keymap.set("n", "<C-t>", function() harpoon_ui.nav_file(2) end)
  vim.keymap.set("n", "<C-n>", function() harpoon_ui.nav_file(3) end)
  vim.keymap.set("n", "<C-s>", function() harpoon_ui.nav_file(4) end)

  local treesitter_highlight = true

  if vim.g.vscode then
    treesitter_highlight = false
  end

  local treesitter_languages = vim.list_extend({ 'c', 'cpp', 'go', 'lua', 'python', 'rust', 'vimdoc', 'vim' },
    languages.treesitter)


  -- [[ Configure Treesitter ]]
  -- See `:help nvim-treesitter`
  require('nvim-treesitter.configs').setup {
    modules = {},
    sync_install = true,
    ignore_install = {},
    -- Add languages to be installed here that you want installed for treesitter
    ensure_installed = treesitter_languages,

    -- Autoinstall languages that are not installed. Defaults to false (but you can change for yourself!)
    auto_install = true,
    highlight = { enable = treesitter_highlight },
    incremental_selection = {
      enable = true,
      keymaps = {
        init_selection = '<c-space>',
        node_incremental = '<c-space>',
        scope_incremental = '<c-s>',
        node_decremental = '<M-space>',
      },
    },
    textobjects = {
      select = {
        enable = false,
        lookahead = true, -- Automatically jump forward to textobj, similar to targets.vim
        keymaps = {
          -- You can use the capture groups defined in textobjects.scm
          ['aa'] = '@parameter.outer',
          ['ia'] = '@parameter.inner',
          ['af'] = '@function.outer',
          ['if'] = '@function.inner',
          ['ac'] = '@class.outer',
          ['ic'] = '@class.inner',
        },
      },
      move = {
        enable = true,
        set_jumps = true, -- whether to set jumps in the jumplist
        goto_next_start = {
          [']m'] = '@function.outer',
          [']]'] = '@class.outer',
        },
        goto_next_end = {
          [']M'] = '@function.outer',
          [']['] = '@class.outer',
        },
        goto_previous_start = {
          ['[m'] = '@function.outer',
          ['[['] = '@class.outer',
        },
        goto_previous_end = {
          ['[M'] = '@function.outer',
          ['[]'] = '@class.outer',
        },
      },
      swap = {
        enable = true,
        swap_next = {
          ['<leader>a'] = '@parameter.inner',
        },
        swap_previous = {
          ['<leader>A'] = '@parameter.inner',
        },
      },
    },
  }

  local format = function()
    vim.lsp.buf.format({ async = true })
  end

  -- Diagnostic keymaps
  vim.keymap.set('n', '[d', vim.diagnostic.goto_prev, { desc = "Go to previous diagnostic message" })
  vim.keymap.set('n', ']d', vim.diagnostic.goto_next, { desc = "Go to next diagnostic message" })
  vim.keymap.set('n', '<leader>e', vim.diagnostic.open_float, { desc = "Open floating diagnostic message" })
  vim.keymap.set('n', '<leader>q', vim.diagnostic.setloclist, { desc = "Open diagnostics list" })

  vim.diagnostic.config({
    underline = true
  })

  -- Format keymap
  vim.keymap.set('n', '<leader>f', format, { desc = "[F]ormat" })

  -- LSP settings.
  --  This function gets run when an LSP connects to a particular buffer.
  local on_attach = function(_, bufnr)
    -- NOTE: Remember that lua is a real programming language, and as such it is possible
    -- to define small helper and utility functions so you don't have to repeat yourself
    -- many times.
    --
    -- In this case, we create a function that lets us more easily define mappings specific
    -- for LSP related items. It sets the mode, buffer and description for us each time.
    local nmap = function(keys, func, desc)
      if desc then
        desc = 'LSP: ' .. desc
      end

      vim.keymap.set('n', keys, func, { buffer = bufnr, desc = desc })
    end

    nmap('K', vim.lsp.buf.hover, 'Hover')
    nmap('<leader>rn', vim.lsp.buf.rename, '[R]e[n]ame')
    nmap('<leader>ca', vim.lsp.buf.code_action, '[C]ode [A]ction')

    nmap('gd', vim.lsp.buf.definition, '[G]oto [D]efinition')
    nmap('gr', require('telescope.builtin').lsp_references, '[G]oto [R]eferences')
    nmap('gI', vim.lsp.buf.implementation, '[G]oto [I]mplementation')
    nmap('<leader>D', vim.lsp.buf.type_definition, 'Type [D]efinition')
    nmap('<leader>ds', require('telescope.builtin').lsp_document_symbols, '[D]ocument [S]ymbols')
    nmap('<leader>ws', require('telescope.builtin').lsp_dynamic_workspace_symbols, '[W]orkspace [S]ymbols')
    local neotest =
        nmap('<leader>tr', function()
          require("neotest").run.run()
        end, '[T]est [R]un')
    nmap('<leader>tf', function()
      require("neotest").run.run(vim.fn.expand("%"))
    end, '[T]est [F]ile')
    nmap('<leader>tds', function()
      require("neotest").summary.toggle()
    end, '[T]est [D]isplay [S]ummary')

    -- See `:help K` for why this keymap


    -- Lesser used LSP functionality
    nmap('gD', vim.lsp.buf.declaration, '[G]oto [D]eclaration')
    nmap('<leader>wa', vim.lsp.buf.add_workspace_folder, '[W]orkspace [A]dd Folder')
    nmap('<leader>wr', vim.lsp.buf.remove_workspace_folder, '[W]orkspace [R]emove Folder')
    nmap('<leader>wl', function()
      print(vim.inspect(vim.lsp.buf.list_workspace_folders()))
    end, '[W]orkspace [L]ist Folders')

    -- Create a command `:Format` local to the LSP buffer
    vim.api.nvim_buf_create_user_command(bufnr, 'Format', format, { desc = 'Format current buffer with LSP' })
  end

  -- Enable the following language servers
  --  Feel free to add/remove any LSPs that you want here. They will automatically be installed.
  --
  --  Add any additional override configuration in the following tables. They will be passed to
  --  the `settings` field of the server config. You must look up that documentation yourself.
  local servers = {
    -- clangd = {},
    -- gopls = {},
    -- pyright = {},
    rust_analyzer = {
      ["rust-analyzer"] = {
        checkOnSave = {
          command = "clippy",
        },
      },
    },
    bufls = {},
    lua_ls = {
      Lua = {
        workspace = { checkThirdParty = false },
        telemetry = { enable = false },
      },
    },
  }
  servers = vim.tbl_deep_extend("keep", servers, languages.mason)

  local settings = vim.tbl_deep_extend("keep", servers, languages.settings)

  -- Setup neovim lua configuration
  require('neodev').setup()

  -- nvim-cmp supports additional completion capabilities, so broadcast that to servers
  local capabilities = vim.lsp.protocol.make_client_capabilities()
  capabilities = require('cmp_nvim_lsp').default_capabilities(capabilities)

  -- Setup mason so it can manage external tooling
  require('mason').setup({
    PATH = "prepend"
  })

  -- local lsp_util = require 'lspconfig.util'
  -- vim.lsp.start({
  --     cmd = { 'terramate-ls' },
  --     filetypes = { 'terramate' },
  --     name = 'terramate-ls',
  --     root_dir = lsp_util.root_pattern('terramate.tm.hcl', '.git'),
  -- })

  -- Ensure the servers above are installed
  local mason_lspconfig = require 'mason-lspconfig'

  mason_lspconfig.setup {
    ensure_installed = vim.tbl_keys(servers),
  }
  function setup_server(server_name)
    require("lspconfig")[server_name].setup {
      capabilities = capabilities,
      on_attach = on_attach,
      settings = settings[server_name],
    }
  end

  mason_lspconfig.setup_handlers {
    setup_server
  }

  for k, v in pairs(languages.settings) do
    setup_server(k)
  end

  completion.post_plugins(completion)

  vim.api.nvim_set_hl(0, "Normal", { bg = "NONE" })

  vim.api.nvim_set_keymap("", ";", "l", { noremap = true })
  vim.api.nvim_set_keymap("", "l", "k", { noremap = true })
  vim.api.nvim_set_keymap("", "k", "j", { noremap = true })
  vim.api.nvim_set_keymap("", "j", "h", { noremap = true })
  vim.api.nvim_set_keymap("", "h", ";", { noremap = true })

  vim.keymap.set("n", "gt", function()
    return "<cmd>ObsidianTags<CR>"
  end, { desc = "List obsidian tags" }
  )

  require 'options'
  -- The line beneath this is called `modeline`. See `:help modeline`
end

require "utils.on_attach"
require "utils.on_load"
require "utils.plugins"

Plugins:add({
  {
    'nvim-telescope/telescope-fzf-native.nvim',
    -- NOTE: If you are having trouble with this installation,
    --       refer to the README for telescope-fzf-native for more instructions.
    build = 'make',
    cond = function()
      return vim.fn.executable 'make' == 1
    end,
  },
  { 'nvim-telescope/telescope.nvim', version = '*', dependencies = { 'nvim-lua/plenary.nvim' } },
})

OnLoad:add(function()
  -- [[ Configure Telescope ]]
  -- See `:help telescope` and `:help telescope.setup()`
  require('telescope').setup {
    defaults = {
      path_display = { "truncate" },
      mappings = {
        i = {
          ['<C-u>'] = false,
          ['<C-d>'] = false,
        },
      },
    },
  }

  -- Enable telescope fzf native, if installed
  pcall(require('telescope').load_extension, 'fzf')

  -- See `:help telescope.builtin`
  vim.keymap.set('n', '<leader>?', require('telescope.builtin').oldfiles, { desc = '[?] Find recently opened files' })
  vim.keymap.set('n', '<leader><space>', require('telescope.builtin').buffers, { desc = '[ ] Find existing buffers' })
  vim.keymap.set('n', '<leader>/', function()
    -- You can pass additional configuration to telescope to change theme, layout, etc.
    require('telescope.builtin').current_buffer_fuzzy_find(require('telescope.themes').get_dropdown {
      winblend = 10,
      previewer = false,
    })
  end, { desc = '[/] Fuzzily search in current buffer' })

  vim.keymap.set('n', '<leader>sf', require('telescope.builtin').find_files, { desc = '[S]earch [F]iles' })
  vim.keymap.set('n', '<leader>sh', require('telescope.builtin').help_tags, { desc = '[S]earch [H]elp' })
  vim.keymap.set('n', '<leader>sw', require('telescope.builtin').grep_string, { desc = '[S]earch current [W]ord' })
  vim.keymap.set('n', '<leader>sg', require('telescope.builtin').live_grep, { desc = '[S]earch by [G]rep' })
  vim.keymap.set('n', '<leader>sk', require('telescope.builtin').keymaps, { desc = '[S]earch [K]eymaps' })
  vim.keymap.set('n', '<leader>sc', require('telescope.builtin').commands, { desc = '[S]earch [C]ommands' })
  vim.keymap.set('n', '<leader>sb', require('telescope.builtin').builtin, { desc = '[S]earch [B]uiltin' })
  vim.keymap.set('n', '<leader>sd', require('telescope.builtin').diagnostics, { desc = '[S]earch [D]iagnostics' })
end)

OnAttach:add(function(_, bufnr)
  Nmap(bufnr, 'gd', require('telescope.builtin').lsp_definitions, '[G]oto [D]efinitions')
  Nmap(bufnr, 'gtd', require('telescope.builtin').lsp_type_definitions, '[G]oto [T]ype [D]efinition')
  Nmap(bufnr, 'gI', require('telescope.builtin').lsp_implementations, '[G]oto [I]mplementations')
  Nmap(bufnr, 'gr', require('telescope.builtin').lsp_references, '[G]oto [R]eferences')
  Nmap(bufnr, '<leader>ds', require('telescope.builtin').lsp_document_symbols, '[D]ocument [S]ymbols')
  Nmap(bufnr, '<leader>ws', require('telescope.builtin').lsp_dynamic_workspace_symbols, '[W]orkspace [S]ymbols')
  Nmap(bufnr, '<leader>st', require('telescope.builtin').treesitter, '[W]orkspace [S]ymbols')
end)

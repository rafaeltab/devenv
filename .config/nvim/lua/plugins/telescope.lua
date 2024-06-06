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
  { 'nvim-telescope/telescope.nvim', branch = "master",  dependencies = { 'nvim-lua/plenary.nvim' } },
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
    pickers = {
      live_grep = {
        additional_args = { '--hidden', '-g', '!node_modules/**', '-g', '!.git/**', },
      },
      find_files = {
        find_command = { 'rg', '--files', '--hidden', '-g', '!node_modules/**', '-g', '!.git/**', },
        mappings = {
          n = {
            ["cd"] = function(prompt_bufnr)
              local selection = require("telescope.actions.state").get_selected_entry()
              local dir = vim.fn.fnamemodify(selection.path, ":p:h")
              require("telescope.actions").close(prompt_bufnr)

              local oil = require("oil")
              oil.open(dir)
            end
          }
        }
      },
      diagnostics = {
        sort_by = "severity"
      }
    }
  }


  -- Enable telescope fzf native, if installed
  pcall(require('telescope').load_extension, 'fzf')

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
  vim.keymap.set('n', '<leader>sr', require('telescope.builtin').resume, { desc = '[S]earch [R]esume' })
  vim.keymap.set('n', '<leader>sc', require('telescope.builtin').commands, { desc = '[S]earch [C]ommands' })
  vim.keymap.set('n', '<leader>sb', require('telescope.builtin').builtin, { desc = '[S]earch [B]uiltin' })
  vim.keymap.set('n', '<leader>sd', function ()require('telescope.builtin').diagnostics({sort_by = "severity", }) end, { desc = '[S]earch [D]iagnostics' })
  vim.keymap.set('n', '<leader>d', function () require('telescope.builtin').diagnostics({bufnr = 0}) end, { desc = '[D]iagnostics' })
end)

OnAttach:add(function(_, bufnr)
  Nmap(bufnr, 'gd', require('telescope.builtin').lsp_definitions, '[G]oto [D]efinitions')
  Nmap(bufnr, 'gtd', require('telescope.builtin').lsp_type_definitions, '[G]oto [T]ype [D]efinition')
  Nmap(bufnr, 'gI', require('telescope.builtin').lsp_implementations, '[G]oto [I]mplementations')
  Nmap(bufnr, 'gr', require('telescope.builtin').lsp_references, '[G]oto [R]eferences')
  Nmap(bufnr, '<leader>ssd', require('telescope.builtin').lsp_document_symbols, '[D]ocument [S]ymbols')
  Nmap(bufnr, '<leader>ssw', require('telescope.builtin').lsp_dynamic_workspace_symbols, '[W]orkspace [S]ymbols')
  Nmap(bufnr, '<leader>st', require('telescope.builtin').treesitter, '[S]earch [T]reesitter')
end)

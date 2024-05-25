require "utils.on_attach"
require "utils.plugins"

local languages = require 'languages.languages'

Plugins:add({
    {
        "aznhe21/actions-preview.nvim"
    },
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
    { 'folke/neodev.nvim', opts = {} }
})

OnLoad:add(function()
    -- Diagnostic keymaps
    vim.keymap.set('n', '[d', vim.diagnostic.goto_prev, { desc = "Go to previous diagnostic message" })
    vim.keymap.set('n', ']d', vim.diagnostic.goto_next, { desc = "Go to next diagnostic message" })
    vim.keymap.set('n', '<leader>e', vim.diagnostic.open_float, { desc = "Open floating diagnostic message" })
    vim.keymap.set('n', '<leader>q', vim.diagnostic.setloclist, { desc = "Open diagnostics list" })


    vim.diagnostic.config({
        underline = true
    })

    local servers = {
        svelte = {},
        html = {},
        -- clangd = {},
        gopls = {},
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
        vale_ls = {
            filetypes = { "markdown", "text" --[[ , "dart" ]] },
            -- filetypes = { "*" },
        }
    }
    servers = vim.tbl_deep_extend("keep", servers, languages.mason)

    local settings = vim.tbl_deep_extend("keep", servers, languages.settings)


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
    local function setup_server(server_name)
        local filetypes = settings[server_name].filetypes
        require("lspconfig")[server_name].setup {
            capabilities = capabilities,
            on_attach = function(client, bufnr) OnAttach:attach(client, bufnr) end,
            settings = settings[server_name],
            filetypes = filetypes
        }
    end

    mason_lspconfig.setup_handlers {
        setup_server
    }

    for k in pairs(languages.settings) do
        setup_server(k)
    end
end)

OnAttach:add(function(_, bufnr)
    Nmap(bufnr, '<leader>ca', require("actions-preview").code_actions, '[C]ode [A]ction')

    Nmap(bufnr, 'K', vim.lsp.buf.hover, 'Hover')
    Nmap(bufnr, '<leader>rn', vim.lsp.buf.rename, '[R]e[n]ame')
    Nmap(bufnr, 'gD', vim.lsp.buf.declaration, '[G]oto [D]eclaration')
end)

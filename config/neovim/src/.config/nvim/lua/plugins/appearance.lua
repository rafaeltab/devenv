local highlight = {
    "RainbowRed",
    "RainbowYellow",
    "RainbowBlue",
    "RainbowOrange",
    "RainbowGreen",
    "RainbowViolet",
    "RainbowCyan",
}

local function show_macro_recording()
    local recording_register = vim.fn.reg_recording()
    if recording_register == "" then
        return ""
    else
        return "Recording @" .. recording_register
    end
end

Plugins:add({
    {
        'lukas-reineke/indent-blankline.nvim',
        main = "ibl",
    },
    {
        'norcalli/nvim-colorizer.lua',
    },
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
            sections = {
                lualine_c = {
                    'filename',
                    { "macro-recording", fmt = show_macro_recording, }
                },
            }
        },
    },
    {
        "MunifTanjim/nui.nvim"
    },
    {
        "folke/noice.nvim",
        event = "VeryLazy",
        opts = {
            lsp = {
                -- override markdown rendering so that **cmp** and other plugins use **Treesitter**
                override = {
                    ["vim.lsp.util.convert_input_to_markdown_lines"] = true,
                    ["vim.lsp.util.stylize_markdown"] = true,
                    ["cmp.entry.get_documentation"] = true, -- requires hrsh7th/nvim-cmp
                },
            },
            -- you can enable a preset for easier configuration
            presets = {
                bottom_search = true, -- use a classic bottom cmdline for search
                command_palette = true, -- position the cmdline and popupmenu together
                long_message_to_split = true, -- long messages will be sent to a split
                inc_rename = false, -- enables an input dialog for inc-rename.nvim
                lsp_doc_border = false, -- add a border to hover docs and signature help
            },
        },
        dependencies = {
            -- if you lazy-load any plugin below, make sure to add proper `module="..."` entries
            "MunifTanjim/nui.nvim",
            -- OPTIONAL:
            --   `nvim-notify` is only needed, if you want to use the notification view.
            --   If not available, we use `mini` as the fallback
            "rcarriga/nvim-notify",
        }
    },
    {
        "rcarriga/nvim-notify",
        opts = {
            background_color = "#000000",
            stages = "static",
        }
    },
})

OnLoad:add(function()
    -- setup notify with a background color so it does not complain
    ---@diagnostic disable-next-line: missing-fields
    require("notify").setup({
        background_colour = "#000000"
    })

    -- Set up rainbow rainbow delimiters
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

        vim.api.nvim_set_hl(0, "Visual", { bg = "#555555" })
        -- Default 535965
        vim.api.nvim_set_hl(0, "Comment", { fg = "#417841" })
        vim.api.nvim_set_hl(0, "@comment", { link = "Comment" })
        vim.api.nvim_set_hl(0, "@lsp.type.comment", { link = "@comment" })
        vim.api.nvim_set_hl(0, "SpecialComment", { link = "Comment" })
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

    -- Highlight yank
    local highlight_group = vim.api.nvim_create_augroup('YankHighlight', { clear = true })
    vim.api.nvim_create_autocmd('TextYankPost', {
        callback = function()
            vim.highlight.on_yank()
        end,
        group = highlight_group,
        pattern = '*',
    })
    local lualine = require("lualine")

    vim.api.nvim_create_autocmd("RecordingEnter", {
        callback = function()
            lualine.refresh({
                place = { "statusline" },
            })
        end,
    })

    vim.api.nvim_create_autocmd("RecordingLeave", {
        callback = function()
            -- This is going to seem really weird!
            -- Instead of just calling refresh we need to wait a moment because of the nature of
            -- `vim.fn.reg_recording`. If we tell lualine to refresh right now it actually will
            -- still show a recording occuring because `vim.fn.reg_recording` hasn't emptied yet.
            -- So what we need to do is wait a tiny amount of time (in this instance 50 ms) to
            -- ensure `vim.fn.reg_recording` is purged before asking lualine to refresh.
            local timer = vim.loop.new_timer()
            timer:start(
                50,
                0,
                vim.schedule_wrap(function()
                    lualine.refresh({
                        place = { "statusline" },
                    })
                end)
            )
        end,
    })
end)

package.path = package.path .. ";" .. vim.fn.stdpath("config") .. "/lua/?.lua"


-- The nvim configuration for nvim
require 'base'

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
        priority = 1000, -- Very high priority is required, luarocks.nvim should run as the first plugin in your config.
        config = true,
    }
})
setup()


vim.cmd.colorscheme 'onedark'

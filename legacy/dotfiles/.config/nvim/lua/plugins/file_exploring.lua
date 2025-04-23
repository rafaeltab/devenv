Plugins:add({
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
})

OnLoad:add(function()
    Map("n", "<leader>pv", function()
        local oil = require("oil")
        oil.open(oil.get_current_dir())
    end, { desc = "Open Oil file manager in directory of current buffer" })
end)

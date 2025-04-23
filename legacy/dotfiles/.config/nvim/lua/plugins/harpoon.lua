Plugins:add({
    {
        'theprimeagen/harpoon',
        opts = {
            menu = {
                width = math.floor(vim.api.nvim_win_get_width(0) / 2),
                height = math.floor(vim.api.nvim_win_get_height(0) / 2),
            }
        }
    },
})
OnLoad:add(function()
    local harpoon_mark = require("harpoon.mark")
    local harpoon_ui = require("harpoon.ui")

    vim.keymap.set("n", "<leader>ha", harpoon_mark.add_file)
    vim.keymap.set("n", "<C-e>", harpoon_ui.toggle_quick_menu)

    vim.keymap.set("n", "<C-h>", function() harpoon_ui.nav_file(1) end)
    vim.keymap.set("n", "<C-t>", function() harpoon_ui.nav_file(2) end)
    vim.keymap.set("n", "<C-n>", function() harpoon_ui.nav_file(3) end)
    vim.keymap.set("n", "<C-s>", function() harpoon_ui.nav_file(4) end)
end)

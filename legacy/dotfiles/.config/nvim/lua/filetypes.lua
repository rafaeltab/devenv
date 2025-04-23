vim.api.nvim_create_autocmd({ "BufNewFile", "BufRead" }, {
    pattern = { "*.arb" },
    callback = function()
        vim.cmd(":set filetype=arb")
    end
})

vim.api.nvim_create_autocmd({ "BufNewFile", "BufRead" }, {
    pattern = { "**/.vscode/**/*.json" },
    callback = function(ev)
        vim.cmd(":set filetype=jsonc")
    end
})

vim.api.nvim_create_autocmd({ "BufNewFile", "BufRead" }, {
    pattern = { "*.bicep" },
    callback = function(ev)
        vim.cmd(":set filetype=bicep")
    end
})

Languages:add_lspconfig(true, "rust_analyzer", {
    ["rust-analyzer"] = {
        checkOnSave = {
            command = "clippy",
        },
    },
    -- root_dir = function(fname)
    --     -- Check for the existence of rust-project.json in the current or parent directories
    --     -- local root = vim.fn.finddir("rust-project.json", vim.fn.expand("%:p:h") .. ";")
    --     --
    --     -- -- If found, return the directory where rust-project.json is located
    --     -- if root ~= "" then
    --     --     return vim.fn.fnamemodify(root, ":h")
    --     -- end
    --
    --     -- If not found, fall back to searching for Cargo.toml (default behavior)
    --     return vim.fn.finddir("Cargo.toml", vim.fn.expand("%:p:h") .. ";") or vim.fn.getcwd()
    -- end
})

Languages:add_treesitter("rust")

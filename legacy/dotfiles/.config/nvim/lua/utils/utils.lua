---@param bufnr integer
---@param keys string
---@param func function
---@param desc string
Nmap = function(bufnr, keys, func, desc)
    if desc then
        desc = 'LSP: ' .. desc
    end

    if not func then
        error("We got an empty func!", 2)
    end

    vim.keymap.set('n', keys, func, { buffer = bufnr, desc = desc })
end

--- @param mode string
--- @param keys string
--- @param rhs string|function
--- @param opts table
Map = function(mode, keys, rhs, opts)
    local options = { noremap = true, silent = true }
    if opts then
        options = vim.tbl_extend("force", options, opts)
    end
    vim.keymap.set(mode, keys, rhs, options)
end

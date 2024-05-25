-- Define the OnAttach class
OnAttach = {
    actions = {}
}

-- Method to add a new action to the list
function OnAttach:add(action)
    table.insert(self.actions, action)
end

-- Method to attach and execute all actions with the given client and bufnr
function OnAttach:attach(client, bufnr)
    for _, action in ipairs(self.actions) do
        action(client, bufnr)
    end
end

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

return {}

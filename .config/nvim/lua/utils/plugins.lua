-- Define the OnAttach class
Plugins = {
    plugins = {}
}

-- Method to add a new action to the list
function Plugins:add(plugin)
    table.insert(self.plugins, plugin)
end

-- Method to attach and execute all actions with the given client and bufnr
function Plugins:get_plugins()
    return self.plugins
end

return {}

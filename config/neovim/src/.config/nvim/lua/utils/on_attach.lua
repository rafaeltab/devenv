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

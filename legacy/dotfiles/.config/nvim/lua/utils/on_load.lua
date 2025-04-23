-- Define the OnAttach class
OnLoad = {
    actions = {}
}

-- Method to add a new action to the list
function OnLoad:add(action)
    table.insert(self.actions, action)
end

function OnLoad:load()
    for _, action in ipairs(self.actions) do
        action()
    end
end

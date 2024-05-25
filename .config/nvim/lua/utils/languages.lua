-- Define the OnAttach class
Languages = {
    mason = {},
    lspconfig = {},
    treesitter = {},
}

--- Method to add language to lspconfig
--- @param mason boolean
--- @param language string
--- @param settings table
function Languages:add_lspconfig(mason, language, settings)
    if mason then
        table.insert(self.mason, language)
    end

   self.lspconfig = vim.tbl_deep_extend("keep", self.lspconfig, {
        [language] = settings
    })
end

function Languages:add_treesitter(language)
    table.insert(self.treesitter, language)
end

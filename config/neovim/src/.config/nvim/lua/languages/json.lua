Plugins:add({
    "b0o/schemastore.nvim",
})

LanguagesV2:configure_language(function()
    --- @type LanguageConfig
    return {
        lspconfig = {
            ['jsonls'] = {
                settings = {
                    json = {
                        schemas = require('schemastore').json.schemas(),
                        validate = { enable = true },
                    }
                }
            }
        },
        mason = { 'jsonls' },
        treesitter = { 'json' }
    }
end)

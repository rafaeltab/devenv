LanguagesV2:configure_language(function()
    --- @type LanguageConfig
    return {
        lspconfig = {
            ['yamlls'] = {
                settings = {
                    schemaStore = {
                        -- You must disable built-in schemaStore support if you want to use
                        -- this plugin and its advanced options like `ignore`.
                        enable = false,
                        -- Avoid TypeError: Cannot read properties of undefined (reading 'length')
                        url = "",
                    },
                    schemas = require('schemastore').yaml.schemas(),
                }
            },
            ['azure_pipelines_ls'] = {}
        },
        mason = { 'yamlls', 'azure_pipelines_ls' },
        treesitter = { 'yaml' }
    }
end)

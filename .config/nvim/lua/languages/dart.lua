LanguagesV2:configure_language(function()
    --- @type LanguageConfig
    return {
        lspconfig = {
            ['dartls'] = {
                settings = {
                    dart = {
                        lineLength = 120
                    }
                }
            }
        },
        mason = { },
        treesitter = { 'dart' }
    }
end)

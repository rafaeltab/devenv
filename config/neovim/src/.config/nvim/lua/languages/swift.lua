LanguagesV2:configure_language(function()
    --- @type LanguageConfig
    return {
        lspconfig = {
            ['sourcekit'] = {}
        },
        mason = {},
        treesitter = { 'swift' }
    }
end)

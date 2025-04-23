LanguagesV2:configure_language(function()
    --- @type LanguageConfig
    return {
        lspconfig = {
            ['gopls'] = {}
        },
        mason = { 'gopls' },
        treesitter = { 'go' }
    }
end)

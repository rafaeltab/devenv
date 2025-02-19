LanguagesV2:configure_language(function()
    --- @type LanguageConfig
    return {
        lspconfig = {
            ['java_language_server'] = {}
        },
        mason = { 'java_language_server' },
        treesitter = { 'java' }
    }
end)

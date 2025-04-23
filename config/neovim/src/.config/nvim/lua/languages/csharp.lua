LanguagesV2:configure_language(function()
    --- @type LanguageConfig
    return {
        lspconfig = {
            ['csharp_ls'] = {
            }
        },
        mason = { 'csharp_ls' },
        treesitter = { 'c_sharp' }
    }
end)

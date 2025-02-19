LanguagesV2:configure_language(function()
    --- @type LanguageConfig
    return {
        lspconfig = {
            ['omnisharp'] = {
                cmd = { "dotnet", "/usr/local/share/omnisharp/OmniSharp.dll" },
            }
        },
        mason = { 'omnisharp' },
        treesitter = { 'c_sharp' }
    }
end)

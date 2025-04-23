LanguagesV2:configure_language(function()
    --- @type LanguageConfig
    return {
        lspconfig = {
            bicep = {
                cmd = { "dotnet", "/usr/local/bin/bicep-langserver/Bicep.LangServer.dll" },
                filetypes = { "bicep" },
            }
        },
        mason = { 'bicep' },
        treesitter = { 'bicep' }
    }
end)

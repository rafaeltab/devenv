LanguagesV2:configure_language(function()
    --- @type LanguageConfig
    return {
        lspconfig = {
            powershell_es = {
                ['powershell_es'] = {}
            }
        },
        mason = { 'powershell_es' },
        treesitter = { 'powershell' }
    }
end)

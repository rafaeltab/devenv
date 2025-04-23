LanguagesV2:configure_language(function()
    --- @type LanguageConfig
    return {
        lspconfig = {
            ['the language server name'] = {
                cmd = { "the command" },
                filetypes = { "the file types" },
            }
        },
        mason = { 'the language server name' },
        treesitter = { 'the language name' }
    }
end)

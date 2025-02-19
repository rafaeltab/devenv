LanguagesV2:configure_language(function()
    --- @type LanguageConfig
    return {
        lspconfig = {
            ['rust_analyzer'] = {
                settings = {
                    ['rust-analyzer'] = {
                        checkOnSave = {
                            command = "clippy"
                        }
                    }
                }
            }
        },
        mason = { 'rust_analyzer' },
        treesitter = { 'rust' }
    }
end)

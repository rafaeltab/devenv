LanguagesV2:configure_language(function()
    --- @type LanguageConfig
    return {
        lspconfig = {
            ['marksman'] = {}
        },
        mason = { 'marksman' },
        treesitter = { 'markdown','markdown_inline' },
    }
end)

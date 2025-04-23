Plugins:add({
    'terramate-io/vim-terramate'
})

LanguagesV2:configure_language(function()
    --- @type LanguageConfig
    return {
        lspconfig = {
            ['terraformls'] = {}
        },
        mason = { 'terraformls' },
        treesitter = { 'terraform' }
    }
end)

LanguagesV2:configure_language(function()
  --- @type LanguageConfig
  return {
    lspconfig = {
      ['eslint'] = {},
      ['tailwindcss'] = {},
      ['ts_ls'] = {},
      ['angularls'] = {},
    },
    mason = { 'eslint', 'tailwindcss', 'ts_ls', 'prettierd', },
    treesitter = { 'tsx', 'typescript' }
  }
end)

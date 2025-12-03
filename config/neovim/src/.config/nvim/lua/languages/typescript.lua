LanguagesV2:configure_language(function()
  --- @type LanguageConfig
  return {
    lspconfig = {
      ['eslint'] = {},
      ['tailwindcss'] = {},
      ['ts_ls'] = {},
      ['angularls'] = {},
      ['biome'] = {},
    },
    mason = { 'eslint', 'tailwindcss', 'ts_ls', 'prettierd', 'biome' },
    treesitter = { 'tsx', 'typescript' }
  }
end)

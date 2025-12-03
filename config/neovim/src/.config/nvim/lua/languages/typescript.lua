LanguagesV2:configure_language(function()
  --- @type LanguageConfig
  return {
    lspconfig = {
      ['eslint'] = {},
      ['tailwindcss'] = {},
      ['ts_ls'] = {},
      ['angularls'] = {
        root_dir = function(fname)
          local util = require 'lspconfig.util'
          local root = util.root_pattern('angular.json')(fname)
          return root
        end,
      },
      ['biome'] = {},
    },
    mason = { 'eslint', 'tailwindcss', 'ts_ls', 'prettierd', 'biome' },
    treesitter = { 'tsx', 'typescript' }
  }
end)

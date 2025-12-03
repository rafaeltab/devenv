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
      ['biome'] = {
        root_dir = function(fname)
          local util = require 'lspconfig.util'
          local root = util.root_pattern('biome.json')(fname)
          return root
        end,
      },
    },
    mason = { 'eslint', 'tailwindcss', 'ts_ls', 'prettierd', 'biome' },
    treesitter = { 'tsx', 'typescript' }
  }
end)

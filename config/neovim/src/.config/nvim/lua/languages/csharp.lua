Plugins:add({
  { "Hoffs/omnisharp-extended-lsp.nvim", lazy = true }
})

LanguagesV2:configure_language(function()
  --- @type LanguageConfig
  return {
    -- lspconfig = {
    --   ['csharp_ls'] = {
    --   }
    -- },
    -- mason = { 'csharp_ls' },
    lspconfig = {
      fsautocomplete = {},
      omnisharp = {
        handlers = {
          ["textDocument/definition"] = function(...)
            return require("omnisharp_extended").handler(...)
          end,
        },
        settings = {
          RoslynExtensionsOptions = {
            EnableAnalyzersSupport = true,
            EnableImportCompletion = false,
            AnalyzeOpenDocumentsOnly = true,
          },
          MsBuild = {
            LoadProjectsOnDemand = true,
          },
          FormattingOptions = {
            EnableEditorConfigSupport = true,
            OrganizeImports = true,
          },
        },
      }
    },
    -- lspconfig = {},
    mason = { "csharpier" },
    treesitter = { 'c_sharp' }
  }
end)

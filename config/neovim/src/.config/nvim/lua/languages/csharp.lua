Plugins:add({
  { "Hoffs/omnisharp-extended-lsp.nvim", lazy = true }
})

LanguagesV2:configure_language(function()
  --- @type LanguageConfig
  return {
    -- lspconfig = {
    --     ['csharp_ls'] = {
    --     }
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
        -- keys = {
        --   {
        --     "gd",
        --     require("omnisharp_extended").telescope_lsp_definitions()
        --     ,
        --     desc = "Goto Definition",
        --   },
        -- },
        enable_roslyn_analyzers = true,
        organize_imports_on_format = true,
        enable_import_completion = true,
      }
    },
    mason = { "csharpier" },
    treesitter = { 'c_sharp' }
  }
end)

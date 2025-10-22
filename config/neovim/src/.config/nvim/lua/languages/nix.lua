LanguagesV2:configure_language(function()
  --- @type LanguageConfig
  return {
    lspconfig = {
      nil_ls = {
        ["nil"] = {
          formatting = { command = { "nixpkgs-fmt" } },
          nix = {
            -- Improve flake/project awareness
            flake = {
              autoArchive = true,
              autoEvalInputs = true,
            },
          },
        },
      }
    },
    mason = { 'nil_ls', 'nixpkgs-fmt' },
    treesitter = { 'nix' }
  }
end)

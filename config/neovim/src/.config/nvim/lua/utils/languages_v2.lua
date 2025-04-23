LanguagesV2 = {
  languages = {},
}

--- Type definition for language configuration
--- @alias LanguageConfiguration fun(): LanguageConfig

--- Method to add language to lspconfig
--- @param configuration LanguageConfiguration
function LanguagesV2:configure_language(configuration)
  table.insert(self.languages, configuration)
end

--- Builds the final configuration table by processing each language configuration.
--- @return { mason: string[], treesitter: string[], lspconfig: table }
function LanguagesV2:build()
  local final_config = {
    mason = {},
    treesitter = {},
    lspconfig = {},
  }

  for _, config_fn in ipairs(self.languages) do
    local config = config_fn()

    -- Add mason packages
    if config.mason then
      for _, package in ipairs(config.mason) do
        table.insert(final_config.mason, package)
      end
    end

    -- Add treesitter languages
    if config.treesitter then
      for _, language in ipairs(config.treesitter) do
        table.insert(final_config.treesitter, language)
      end
    end

    -- Extend lspconfig settings
    if config.lspconfig then
      for language, settings in pairs(config.lspconfig) do
        final_config.lspconfig = vim.tbl_deep_extend(
          "keep",
          final_config.lspconfig,
          {
            [language] = settings,
          }
        )
      end
    end
  end

  return final_config
end

return {
  lsp = {
    use = 'mason',
    mason = {
      ["terraformls"] = {},
    }
  },
  plugins = { 'terramate-io/vim-terramate' },
  treesitter = { 'terraform' }
}

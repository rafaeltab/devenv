-- The nvim configuration for nvim
require 'base'
setup({
  {
    'declancm/cinnamon.nvim',
    config = function()
      require('cinnamon').setup({
        default_keymaps = true,
        extra_keymaps = true,
        extended_keymaps = true,
        always_scroll = true,
        centered = true,
        scroll_limit = 256
      })
    end
  },
  {
    -- Theme inspired by Atom
    'martinsione/darkplus.nvim',
    priority = 1000,
    config = function()
      vim.cmd.colorscheme 'darkplus'
    end,
  }
})
vim.opt.termguicolors = true
-- vim: ts=2 sts=2 sw=2 et

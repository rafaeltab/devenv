require "utils.on_load"

OnLoad:add(function()
  -- Make sure space does nothing as it is our leader
  vim.keymap.set({ 'n', 'v' }, '<Space>', '<Nop>', { silent = true })

  vim.api.nvim_set_keymap("", ";", "l", { noremap = true })
  vim.api.nvim_set_keymap("", "l", "k", { noremap = true })
  vim.api.nvim_set_keymap("", "k", "j", { noremap = true })
  vim.api.nvim_set_keymap("", "j", "h", { noremap = true })
  vim.api.nvim_set_keymap("", "h", ";", { noremap = true })

  vim.keymap.set({ 'n' }, 'd', '"_d')
  vim.keymap.set({ 'n', 'v' }, 'x', '"_x')
  vim.keymap.set({ 'n', 'v' }, 'c', '"_c')

  vim.keymap.set({ 'n' }, '<leader>w', ':w<CR>', { desc = "Save the current file" })

  -- exit terminal mode
  vim.keymap.set({ 't' }, '<Esc>', '<C-\\><C-n>')
end)

require "utils.on_load"
require "utils.casing"

--- @param alias string
--- @param command string
local function add_alias(alias, command)
  vim.api.nvim_create_user_command(alias, function()
    vim.cmd(command)
  end, { desc = command })
end

OnLoad:add(function()
  -- Make sure space does nothing as it is our leader
  vim.keymap.set({ 'n', 'v' }, '<Space>', '<Nop>', { silent = true })

  vim.api.nvim_set_keymap("", ";", "l", { noremap = true })
  vim.api.nvim_set_keymap("", "l", "k", { noremap = true })
  vim.api.nvim_set_keymap("", "k", "j", { noremap = true })
  vim.api.nvim_set_keymap("", "j", "h", { noremap = true })
  vim.api.nvim_set_keymap("", "h", ";", { noremap = true })

  vim.api.nvim_set_keymap("n", "<C-W>;", "<C-W>l", { noremap = true, desc = "Go to the right window" })
  vim.api.nvim_set_keymap("n", "<C-W>l", "<C-W>k", { noremap = true, desc = "Go to the bottom window" })
  vim.api.nvim_set_keymap("n", "<C-W>k", "<C-W>j", { noremap = true, desc = "Go to the top window" })
  vim.api.nvim_set_keymap("n", "<C-W>j", "<C-W>h", { noremap = true, desc = "Go to the left window" })
  vim.api.nvim_set_keymap("n", "<C-W>h", "<Nop>", { noremap = true, silent = true })

  vim.keymap.set({ 'n' }, 'd', '"_d')
  vim.keymap.set({ 'n', 'v' }, 'x', '"_x')
  vim.keymap.set({ 'n', 'v' }, 'c', '"_c')

  -- Visual block mode
  vim.keymap.set({ 'n', 'x' }, '<leader>v', '<c-q>')

  vim.keymap.set({ 'n' }, '<leader>w', ':w<CR>', { desc = "Save the current file" })

  -- exit terminal mode
  vim.keymap.set({ 't' }, '<Esc>', '<C-\\><C-n>')

  add_alias("Ga", "Git add -A")
  add_alias("Gc", "Git commit")
  add_alias("Gca", "Git commit --amend")
  add_alias("Gp", "Git push")
  add_alias("Gpn", "Git push --no-verify")
  add_alias("Gpfn", "Git push --force --no-verify")
  add_alias("Gpf", "Git push --force")

  setupCaseBindings()
end)

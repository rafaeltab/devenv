vim.api.nvim_create_autocmd({ "BufNewFile", "BufRead" }, {
  pattern = { "*.arb" },
  callback = function(ev)
    vim.cmd(":set filetype=arb")
  end
})

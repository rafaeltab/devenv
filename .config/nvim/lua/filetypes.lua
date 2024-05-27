-- " my filetype file
-- 	if exists("did_load_filetypes")
-- 	  finish
-- 	endif
-- 	augroup filetypedetect
-- 	  au! BufRead,BufNewFile *.mine		setfiletype mine
-- 	  au! BufRead,BufNewFile *.xyz		setfiletype drawing
-- 	augroup END

vim.api.nvim_create_autocmd({ "BufNewFile", "BufRead" }, {
  pattern = { "*.arb" },
  callback = function(ev)
    vim.cmd(":set filetype=arb")
  end
})

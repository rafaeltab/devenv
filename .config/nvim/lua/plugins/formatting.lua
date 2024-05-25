local format = function()
	vim.lsp.buf.format({ async = true })
end

OnLoad:add(function ()
	vim.keymap.set('n', '<leader>f', format, { desc = "[F]ormat" })
end)

OnAttach:add(function(_, bufnr)
	-- Create a command `:Format` local to the LSP buffer
	vim.api.nvim_buf_create_user_command(bufnr, 'Format', format, { desc = 'Format current buffer with LSP' })
end)

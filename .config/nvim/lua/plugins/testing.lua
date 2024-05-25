require "utils.on_attach"
require "utils.plugins"

Plugins:add({
	{
		"nvim-neotest/neotest",
		dependencies = {
			"nvim-lua/plenary.nvim",
			"nvim-treesitter/nvim-treesitter",
			"antoinemadec/FixCursorHold.nvim",
			'sidlatau/neotest-dart',
			'rouge8/neotest-rust',
			'nvim-neotest/neotest-go',
			'nvim-neotest/nvim-nio'
		},
		config = function()
			require("neotest").setup({
				adapters = {
					require("neotest-dart") {
						command = "flutter",
						use_lsp = true,
						custom_test_method_names = { "blocTest" }
					},
					require("neotest-rust"),
					require("neotest-go")
				},
				consumers = { require("neotest").diagnostic, require("neotest").status }
			})
		end
	},
})
OnAttach:add(function(_, bufnr)
	Nmap(bufnr, '<leader>tr', function()
		require("neotest").run.run()
	end, '[T]est [R]un')
	Nmap(bufnr, '<leader>tf', function()
		require("neotest").run.run(vim.fn.expand("%"))
	end, '[T]est [F]ile')
	Nmap(bufnr, '<leader>tds', function()
		require("neotest").summary.toggle()
	end, '[T]est [D]isplay [S]ummary')
end)

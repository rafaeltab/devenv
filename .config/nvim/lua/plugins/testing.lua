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
            'Issafalcon/neotest-dotnet',
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
                    require("neotest-go"),
                    require("neotest-dotnet"),
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


OnLoad:add(function()
	local notify = require('notify')
	-- Function to get the test file path for the current buffer
	local function get_test_file_path()
		local buf_name = vim.api.nvim_buf_get_name(0) -- Get the current buffer name
		local file_type = vim.bo.filetype         -- Get the current file type

		-- Function to handle Dart test file path
		local function handle_dart()
			-- Check if the current file is already a test file
			if buf_name:find("test/") and buf_name:find("_test.dart$") then
				notify("Current file is already a test file", "warn")
				return nil
			end

			local test_path = buf_name:gsub("lib/", "test/"):gsub(".dart$", "_test.dart")
			return test_path
		end

		-- Switch case for different languages
		if file_type == "dart" then
			return handle_dart()
		else
			notify("Language not supported", "error")
			return nil
		end
	end

	-- Function to open or create the test file
	local function open_or_create_test_file()
		local test_file_path = get_test_file_path()
		if not test_file_path then
			return
		end

		-- Get the directory of the test file path
		local test_file_dir = test_file_path:match("(.*/)")

		-- Check if the directory exists, if not, create it
		if vim.fn.isdirectory(test_file_dir) == 0 then
			local success = vim.fn.mkdir(test_file_dir, "p")
			if success ~= 1 then
				notify("Failed to create directory: " .. test_file_dir, "error")
				return
			end
		end

		-- Check if the test file is already open in another buffer
		for _, buf in ipairs(vim.fn.getbufinfo({ bufloaded = 1 })) do
			if vim.fn.bufname(buf.bufnr) == test_file_path then
				vim.api.nvim_set_current_buf(buf.bufnr)
				notify("Switched to existing buffer for test file", "info")
				return
			end
		end

		-- Open the file (create if it doesn't exist)
		vim.api.nvim_command('edit ' .. test_file_path)
		notify("Opened test file: " .. test_file_path, "info")
	end

	vim.keymap.set({ 'n' }, '<leader>ct', function()
		open_or_create_test_file()
	end, { desc = "[C]reate [T]est" })
end)

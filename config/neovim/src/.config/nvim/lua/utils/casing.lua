function PascalToSnakeCase()
  local current_line_num = vim.fn.line(".")
  local current_line_text = vim.fn.getline(current_line_num)
  local start_byte_col = 0 -- 0-indexed byte column for nvim_buf_set_text
  local end_byte_col = 0   -- 0-indexed byte column for nvim_buf_set_text
  local selected_word = ""

  local mode = vim.fn.mode()

  if mode ~= "v" and mode ~= "V" and mode ~= "\22" then
    print("You're not in visual mode")
  end
  -- Visual selection is active
  local visual_start_pos = vim.fn.getpos("'<") -- [1]lnum, [2]col, [3]vcol
  local visual_end_pos = vim.fn.getpos("'>")   -- [1]lnum, [2]col, [3]vcol

  if visual_start_pos[1] ~= visual_end_pos[1] then
    print("Error: Please select a word within a single line.")
    return
  end

  -- virtcol2col takes (bufnr, lnum, vcol) and returns 1-indexed byte col
  -- nvim_buf_set_text expects 0-indexed byte col
  start_byte_col = vim.fn.virtcol2col(0, visual_start_pos[1], visual_start_pos[3]) - 1
  end_byte_col = vim.fn.virtcol2col(0, visual_end_pos[1], visual_end_pos[3])

  -- The range for string:sub is 1-indexed.
  -- So, if start_byte_col is 0-indexed, add 1.
  -- If end_byte_col is 0-indexed exclusive end, it's already correct.
  selected_word = current_line_text:sub(start_byte_col + 1, end_byte_col)

  if not selected_word or selected_word == "" then
    print("No word selected or under cursor.")
    return
  end

  -- Convert PascalCase to snake_case
  local snake_case_word = string.gsub(selected_word, "([A-Z])", function(c)
    return "_" .. string.lower(c)
  end)
  -- Remove leading underscore if the original word started with an uppercase letter
  if string.sub(snake_case_word, 1, 1) == "_" and #selected_word > 0 and selected_word:sub(1, 1):match("[A-Z]") then
    snake_case_word = string.sub(snake_case_word, 2)
  end

  -- Replace the word in the buffer
  -- vim.api.nvim_buf_set_text uses 0-indexed line and column numbers,
  -- and the end column is EXCLUSIVE.
  vim.api.nvim_buf_set_text(
    0,                    -- current buffer
    current_line_num - 1, -- 0-indexed line number
    start_byte_col,       -- 0-indexed start column (inclusive)
    current_line_num - 1, -- 0-indexed line number
    end_byte_col,         -- 0-indexed end column (exclusive)
    { snake_case_word }
  )

  print("Converted '" .. selected_word .. "' to '" .. snake_case_word .. "'")
end

function setupCaseBindings()
  -- Map it to a key (example: <leader>ps)
  vim.api.nvim_set_keymap("n", "<leader>ps", ":lua PascalToSnakeCase()<CR>",
    { noremap = true, silent = true, desc = "Switch PascalCase to snake_case" })
  vim.api.nvim_set_keymap("v", "<leader>ps", ":lua PascalToSnakeCase()<CR>",
    { noremap = true, silent = true, desc = "Switch PascalCase to snake_case" })
end

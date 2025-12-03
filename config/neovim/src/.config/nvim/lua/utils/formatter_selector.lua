local M = {}

-- Cache for package.json data to avoid re-reading files
-- Structure: { [package_json_path] = { mtime = number, has_biome = bool, has_prettier = bool } }
local cache = {}

-- Find package.json by walking up the directory tree
-- @param filepath string: The current file path
-- @return string|nil: Path to package.json or nil if not found
local function find_package_json(filepath)
  if not filepath or filepath == "" then
    return nil
  end

  local current_dir = vim.fn.fnamemodify(filepath, ":p:h")
  local home_dir = vim.loop.os_homedir()

  while current_dir and current_dir ~= "/" and current_dir ~= home_dir do
    local package_json = current_dir .. "/package.json"
    local stat = vim.loop.fs_stat(package_json)

    if stat and stat.type == "file" then
      return package_json
    end

    -- Move up one directory
    local parent = vim.fn.fnamemodify(current_dir, ":h")
    if parent == current_dir then
      break
    end
    current_dir = parent
  end

  return nil
end

-- Parse package.json and extract dependencies
-- @param package_json_path string: Path to package.json
-- @return table|nil: Table with dependencies and devDependencies, or nil on error
local function parse_package_json(package_json_path)
  local file = io.open(package_json_path, "r")
  if not file then
    return nil
  end

  local content = file:read("*all")
  file:close()

  local ok, decoded = pcall(vim.json.decode, content)
  if not ok then
    return nil
  end

  return {
    dependencies = decoded.dependencies or {},
    devDependencies = decoded.devDependencies or {}
  }
end

-- Check if any of the dependency names exist in package.json
-- @param package_json_path string: Path to package.json
-- @param dep_names table: Array of dependency names to check
-- @return boolean: True if any dependency is found
local function has_dependency(package_json_path, dep_names)
  local parsed = parse_package_json(package_json_path)
  if not parsed then
    return false
  end

  for _, dep_name in ipairs(dep_names) do
    if parsed.dependencies[dep_name] or parsed.devDependencies[dep_name] then
      return true
    end
  end

  return false
end

-- Get cached formatter info or read from package.json
-- @param package_json_path string: Path to package.json
-- @return table: { has_biome = bool, has_prettier = bool }
local function get_formatter_info(package_json_path)
  -- Check if we have valid cache
  local stat = vim.loop.fs_stat(package_json_path)
  if not stat then
    return { has_biome = false, has_prettier = false }
  end

  local cached = cache[package_json_path]
  if cached and cached.mtime == stat.mtime.sec then
    return { has_biome = cached.has_biome, has_prettier = cached.has_prettier }
  end

  -- Cache miss or stale cache, read package.json
  local has_biome = has_dependency(package_json_path, { "@biomejs/biome", "biome" })
  local has_prettier = has_dependency(package_json_path, { "prettier", "prettierd" })

  -- Update cache
  cache[package_json_path] = {
    mtime = stat.mtime.sec,
    has_biome = has_biome,
    has_prettier = has_prettier
  }

  return { has_biome = has_biome, has_prettier = has_prettier }
end

-- Select the appropriate formatter based on package.json dependencies
-- @param filepath string: The current file path
-- @return table: Array of formatter names (e.g., { "biome" }, { "prettierd" }, or {})
function M.select_formatter(filepath)
  local package_json_path = find_package_json(filepath)

  -- No package.json found, use LSP only
  if not package_json_path then
    vim.notify("no package json")
    return {}
  end

  local info = get_formatter_info(package_json_path)

  -- Both formatters found - conflict!
  if info.has_biome and info.has_prettier then
    vim.notify(
      "Both biome and prettier found in package.json. Please use only one formatter. Falling back to LSP formatting.",
      vim.log.levels.ERROR
    )
    return {}
  end

  -- Biome found
  if info.has_biome then
    vim.notify("biome")
    return { "biome" }
  end

  -- Prettier found
  if info.has_prettier then
    vim.notify("prettierd")
    return { "prettierd" }
  end

  vim.notify("neither")
  -- Neither found, use LSP only
  return {}
end

return M

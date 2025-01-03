Languages:add_lspconfig(true, "bicep", {
    cmd = { "dotnet", "/usr/local/bin/bicep-langserver/Bicep.LangServer.dll" };
    filetypes = { "bicep" };
})

Languages:add_treesitter("bicep")

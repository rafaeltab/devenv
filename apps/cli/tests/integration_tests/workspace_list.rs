use crate::common::helpers;

#[test]
pub fn test_cli_integration() {
    let input = r#"{
  "workspaces": [
    {
      "root": "~/dotfiles",
      "id": "dotfiles",
      "name": "Dotfiles",
      "tags": [
        "dotfiles",
        "lua"
      ]
    },
    {
      "root": "~/home/notes/coding_knowledge",
      "id": "coding_knowledge",
      "name": "Notes",
      "tags": [
        "notes",
        "markdown"
      ]
    },
    {
      "root": "~/home/source/rafaeltab",
      "id": "rafaeltab_cli",
      "name": "Rafaeltab cli",
      "tags": [
        "rafaeltab",
        "rust"
      ]
    },
    {
      "root": "~/home/source/code_analyzer",
      "id": "code_analyzer",
      "name": "Code analyzer",
      "tags": [
        "rust"
      ]
    }
  ],
  "tmux": {
    "sessions": [
      {
        "windows": [
          {
            "name": "Neovim",
            "command": "nvim ."
          }
        ],
        "workspace": "coding_knowledge",
        "name": "John"
      }
    ],
    "defaultWindows": [
      {
        "name": "Neovim",
        "command": "nvim ."
      },
      {
        "name": "Zsh",
        "command": null
      }
    ]
  }
}
        "#;

    let expected = r#"Dotfiles (dotfiles): /root/dotfiles ["dotfiles", "lua"]
Notes (coding_knowledge): /root/home/notes/coding_knowledge ["notes", "markdown"]
Rafaeltab cli (rafaeltab_cli): /root/home/source/rafaeltab ["rafaeltab", "rust"]
Code analyzer (code_analyzer): /root/home/source/code_analyzer ["rust"]
"#;

    helpers::setup_test_file(input);
    let (stdout, _stderr) = helpers::run_cli_with_stdin(&["workspace", "list"], "");
    helpers::verify_output(expected, &stdout);

    // helpers::cleanup();
}

# CLI

This workspace contains a cli used for managing workspaces, and tmux sessions.

## Configuration

The CLI reads configuration from `~/.rafaeltab.json` by default.

### Custom Config Path

You can specify a custom config file using the `--config` flag:

```bash
rafaeltab --config /path/to/custom/config.json workspace list
```

This is useful for:

- Testing without affecting your main config
- Managing multiple config profiles
- Running the CLI in CI/CD environments

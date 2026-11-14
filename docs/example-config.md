# Sample configuration

`~/.zaino/config.toml`:

```toml
# Any alias from the bundled catalog: fast, smart, frontier, fable.
model = "smart"

# when to ask before running a command
approval_policy = "on-request"

# what the agent may touch without asking
sandbox_mode = "workspace-write"

[sandbox_workspace_write]
network_access = false
```

Project overrides go in `./.zaino/config.toml` and win over the user file.

The provider is fixed to `zaino` unless you explicitly select another one with
`model_provider`. API keys belong in `ZAINO_API_KEY`, not in this file.

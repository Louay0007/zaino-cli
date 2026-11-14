# Getting started with Zaino CLI

```shell
git clone https://github.com/Louay0007/zaino-cli.git
cd zaino-cli/codex-rs && cargo build --release -p codex-cli
export ZAINO_API_KEY=zn-...
cd your-project
/path/to/zaino-cli/codex-rs/target/release/zaino
```

The npm package (`@zaino/cli`) is not published yet; build from source until it is.

Interactive mode opens a prompt where you can describe a task; Zaino reads and edits files in
the current directory, runs commands in a sandbox and asks before anything destructive.

Common entries:

```shell
zaino                       # interactive TUI
zaino "explain this repo"   # interactive, with a first prompt
zaino exec "run the tests"  # non-interactive, prints the result
zaino login --with-api-key  # store a key
zaino --help                # all subcommands
```

## Models

| Alias | Use |
| --- | --- |
| `fast` | Cheap, quick edits |
| `smart` | Default; everyday coding |
| `frontier` | Hardest problems |
| `fable` | Long-form and explanatory work |

Switch inside the session with `/model`, or per run with `zaino --model frontier`.

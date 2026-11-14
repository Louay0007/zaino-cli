<p align="center"><strong>Zaino CLI</strong> is a coding agent that runs locally on your computer.
</p>

---

## Quickstart

### Installing and running Zaino CLI

Build from source (the npm package is not published yet):

```shell
git clone https://github.com/Louay0007/zaino-cli.git
cd zaino-cli/codex-rs
cargo build --release -p codex-cli
./target/release/zaino --version
```

Then run `zaino` to get started.

### Using Zaino with an API key

Create a key at <https://zaino.soon.it>, then:

```shell
zaino login --with-api-key
```

Paste your `zn-` key and press Enter. The key is stored in `$ZAINO_HOME/.env` (mode `600`), and `zaino logout` removes it.

You can also export it yourself:

```shell
export ZAINO_API_KEY=zn-...
```

No config file is needed: the CLI defaults to the `zaino` provider, which speaks the Responses API at `https://api.zaino.soon.it/v1`. `~/.zaino/config.toml` is optional and only overrides defaults (for example `model = "frontier"`).

### Models

Four aliases, chosen with/model picker or `--model`:

| Alias | Use |
| --- | --- |
| `fast` | Cheap, quick edits |
| `smart` | Default; everyday coding |
| `frontier` | Hardest problems |
| `fable` | Long-form and explanatory work |

### From source

```shell
cd codex-rs
cargo build --release -p codex-cli
./target/release/zaino --version
```

## Docs

- [**Getting started**](./docs/getting-started.md)
- [**Authentication**](./docs/authentication.md)
- [**Configuration**](./docs/config.md)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).

# Making the Zaino CLI work — the plan

> **Superseded.** This is the 2026-11-13 assessment, kept for the record. All
> three blockers are closed: `rust-v1.0.1` is published, and
> `npm install -g zainocli` works on macOS and Linux. The packages ship unscoped
> (`zainocli`, `zainocli-sdk`, `zainocli-responses-api-proxy`) because the
> `@zaino` scope belongs to another account. See `PUBLISH.md` for the current
> process.

**Status at 2026-11-13.** The CLI is a rebranded Codex fork. The agent, the
provider wiring, the model catalog and the on-disk key handling are all correct
and point at `api.zaino.soon.it/v1`. The **distribution layer** was never
re-pointed, and three things block a customer from ever running it.

---

## The three blockers, in order

| # | Blocker | Evidence | Blocks |
|---|---|---|---|
| 1 | Nothing is published | `@zaino/cli` → 404 on npm; `Louay0007/zaino-cli` → 404 on GitHub; `zaino.soon.it/api/downloads` → `{"version":"1.1.5","files":[]}` | Install at step 1 of the README |
| 2 | Tagged releases are impossible | `codex-rs/Cargo.toml` says `version = "0.0.0"`; `rust-release.yml:34` validates tag == Cargo.toml and exits on mismatch | The publish pipeline |
| 3 | The workspace won't build without Chromium | `code-mode-host` (unreferenced by anything) pulls `rusty_v8`, requiring 20 Chromium submodules (ICU, LLVM libcxx, fast_float) | Any local `cargo build -p codex-cli` |

Blockers 1 and 2 must be fixed before anything else matters. Blocker 3 only
affects the developer experience; CI has network and disk to spare, so it is the
least urgent of the three — but it is why this machine cannot produce a binary.

---

## Step 1 — Unblock the build (done, needs deciding)

`codex-code-mode-host` is a **workspace member that no crate depends on**. It is
the only path to `rusty_v8` in the graph. Two options:

**(a) Verified working — remove it from the workspace.** Removing
`"code-mode-host"` and `"v8-poc"` from `[workspace] members` makes
`cargo build -p codex-cli` resolve instantly and compile. This is what was tested
here.

**Consequence:** `just code-mode-host`, `just bazel-code-mode-host`, the winget
package's `codex-code-mode-host.exe` rename, and `scripts/codex_package/layout.py`
all expect that binary. Do **(a)** only if code mode is not shipping. Otherwise…

**(b) Keep it, and stop resolving it for CLI builds.** Add
`[workspace] default-members = [...]` minus the v8 crates, or build the CLI from
a trimmed workspace manifest in CI. This keeps `just` recipes working and costs
one CI job to maintain.

**Recommendation: (a),** then delete the dead `just`/packaging references. If code
mode is on the roadmap, take (b) instead — but do not ship the CLI without
deciding, because either way a customer who clones the repo today cannot build it.

## Step 2 — Set a real version

`[workspace.package] version = "0.0.0"` must become the first release line, e.g.
`1.0.0`. Two consequences:

- `is_source_build_version()` (`tui/src/update_versions.rs:18`) treats `0.0.0` as
  a source build and **disables the update check entirely**. A `0.0.0` binary can
  never be told to update — which is fine now, and wrong the moment you publish.
- `rust-release.yml` refuses to release unless the tag equals `Cargo.toml`.

`v1.1.5` is already taken by the **desktop app**. Pick a CLI-specific version
(`1.0.0` is cleanest) so the two products cannot be confused in support.

## Step 3 — Publish

The release workflow is already rebranded end to end: it stages
`@zaino/cli` plus the six platform variants (`@zaino/cli-darwin-arm64`, …),
publishes under platform dist-tags, and points the meta package at them.

Prerequisites:

1. **Create the GitHub repo** `Louay0007/zaino-cli` (every URL the binary emits —
   update check, release notes, announcement tips — now points there).
2. **Create the npm org `zaino`** and an `NPM_TOKEN` secret, or the `npm publish`
   step fails at auth.
3. Tag `rust-v1.0.0` after Step 2, and let `rust-release.yml` build the six
   platform binaries.
4. Verify: `npm install -g @zaino/cli && zaino --version` on macOS arm64, macOS
   x64, Linux x64.

Until step 3 lands, the README says **build from source** and that is honest.

## Step 4 — Point `zaino app` at a real installer

`zaino app` downloads the desktop DMG. It previously fetched OpenAI's
(`persistent.oaistatic.com`) and verified OpenAI's Team ID; that now points at
`zaino.soon.it/download`, but **no installers are hosted** — `/api/downloads`
returns an empty file list. Either drop `zaino app` from this release, or place
the six installers under `DOWNLOADS_DIR/v1.1.5/` on the VPS so the endpoint lists
them.

The macOS bundle identifier is `com.zaino.cli` and the download page is live, so
the product side is ready; only the files are missing.

---

## What is already correct (do not redo)

- Default provider `zaino` → `https://api.zaino.soon.it/v1`, Responses API
- `ZAINO_API_KEY` / `zn-` keys; `login --with-api-key` → `$ZAINO_HOME/.env` at
  mode 600, read back by `load_dotenv()` in `arg0`
- Model catalog: `fast`, `smart`, `frontier`, `fable` — all four exist in
  `config/litellm.yaml`
- Help text, subcommand descriptions, onboarding sign-in screen
- `doctor` checks, including `com.zaino.cli` identity paths

## Verification that does not need a VPS

```bash
cd codex-rs && cargo build -p codex-cli --offline   # must reach a binary
./target/debug/zaino --version
./target/debug/zaino --help
./target/debug/zaino doctor
ZAINO_API_KEY=zn-… ./target/debug/zaino exec "reply with ok"
```

The last command is the only real end-to-end test: it proves the CLI reaches the
gateway through the responses shim and gets a streamed answer back.

### What has been run (2026-11-13)

Built and executed locally after the workspace fix:

| Command | Result |
|---|---|
| `cargo build -p codex-cli` | binary links in ~18 min cold, ~20 s incremental |
| `zaino --version` | `zaino 0.0.0` (was `codex-cli 0.0.0`) |
| `zaino --help` / `exec --help` | all usage lines say `zaino` |
| `zaino doctor --summary` | passes; exits 0 |
| `zaino login --with-api-key` | writes `$ZAINO_HOME/.env`, mode `600` |
| `zaino logout` | removes it |
| `zaino mcp list` | correct empty state |
| `zaino completion bash` | emits `_zaino()` |
| `zaino features list` / `debug models` | four Zaino aliases, Zaino-branded |
| `zaino update` | debug builds refuse, as designed |
| `zaino exec` | **reaches `api.zaino.soon.it/v1/responses`**; the edge translates the `zn-` key to `sk-` correctly. It fails only on the credential in this shell, which LiteLLM rejects as revoked — not a CLI bug. |

**The one test still outstanding** is a successful completion with a live `zn-`
key. Every layer up to LiteLLM's token lookup is proven working; mint a fresh key
in the portal and rerun the `exec` line above.

`zaino agents` and the bare TUI need a TTY, so they were not exercised here.

# Publishing `@zaino/cli` to npm

The CLI is integration-complete: it logs in through the portal's device flow,
reports the allowance from `/api/device/usage`, and ships a model catalog that
matches what the gateway actually serves. What is left is distribution, and it
needs credentials only you have.

---

## What is already done

| Step | State |
|---|---|
| Version stamped `1.0.0` in `codex-rs/Cargo.toml` | done |
| Meta package stages correctly | verified — `build_npm_package.py --package zaino --version 1.0.0` |
| Platform package matrix (6 targets) | defined in the build script |
| Release workflow points at npm + OIDC | done, unchanged from the fork |
| Device login against the live portal | verified — issues a code, polls `authorization_pending` correctly |
| Usage reporting in `login status` | verified — degrades silently on a bad key |
| 308/308 CLI tests | pass |

---

## The one thing blocking: no npm identity

`npm whoami` fails on this machine. Publishing needs an authenticated npm
account that owns (or belongs to) the `@zaino` scope. Everything below is
blocked on that.

```bash
npm login          # or: npm adduser
npm whoami         # must print your username
```

Then create the scope once, in the npm web UI — an organisation named `zaino`:

- <https://www.npmjs.com/org/create>

Two more things on that page before the packages can be trusted to publish
automatically:

1. Add `@zaino/cli`, `@zaino/cli-darwin-arm64`, … (all six) to the org.
2. Under each package's **Settings → Trusted publishing**, link this repository
   (`Louay0007/zaino-cli`) and the workflow (`.github/workflows/rust-release.yml`).
   The release job uses OIDC, not an `NPM_TOKEN`, so without that link the
   publish step fails with 403.

---

## Build and publish, in order

The meta package resolves the platform binaries through `optionalDependencies`,
so **platform packages must be published first** — npm will otherwise serve a
meta package that cannot install.

```bash
# 1. Build the release binaries for each target you intend to ship.
#    On this machine only darwin-arm64 can be built; the other five need CI
#    (or a cross-compile toolchain).
cd codex-rs
cargo build --release -p codex-cli

# 2. Assemble the vendor tree the packaging script expects:
#    vendor/<target-triple>/bin/zaino
mkdir -p /tmp/zaino-vendor/aarch64-apple-darwin/bin
cp target/release/zaino /tmp/zaino-vendor/aarch64-apple-darwin/bin/zaino
cat > /tmp/zaino-vendor/aarch64-apple-darwin/codex-package.json <<'JSON'
{
  "layoutVersion": 1,
  "version": "1.0.0",
  "target": "aarch64-apple-darwin",
  "variant": "codex",
  "entrypoint": "bin/zaino"
}
JSON

# 3. Stage and pack the platform package (python3.12 — the script uses 3.10+ syntax).
cd ..
python3.12 codex-cli/scripts/build_npm_package.py \
  --package zaino-darwin-arm64 \
  --version 1.0.0 \
  --vendor-src /tmp/zaino-vendor \
  --staging-dir /tmp/stage-darwin-arm64

python3.12 codex-cli/scripts/build_npm_package.py \
  --package zaino \
  --version 1.0.0 \
  --staging-dir /tmp/stage-meta
```

Then publish each staged directory with `npm publish /tmp/stage-<name> --tag latest`,
platform packages first.

**The honest path is CI.** Tag `rust-v1.0.0` and push; `.github/workflows/rust-release.yml`
builds all six targets, signs and notarizes macOS, and publishes through OIDC.
That is the only way to get a complete set of packages, and it is already written.

---

## What is still missing after a publish

| Gap | Effect | Fix |
|---|---|---|
| No GitHub repo `Louay0007/zaino-cli` | every URL the binary emits (update check, release notes, announcement tips) 404s | create the repo and push |
| macOS binaries unsigned | Gatekeeper blocks the download; users see a security warning | Apple Developer Program ($99/yr) + the `AKV_*` secrets |
| Installers not hosted | `zaino app` has nothing to fetch | place the DMGs in `DOWNLOADS_DIR/v1.1.5/` on the VPS |
| Code mode removed from the build | that feature does not ship | re-add `code-mode-host` with `default-members` instead |
| No Homebrew tap | doctor's `brew upgrade --cask zaino` would fail | either publish a cask or drop the advice |

---

## Verify a publish actually works

On a clean machine, with no checkout:

```bash
npm install -g @zaino/cli
zaino --version                 # must print: zaino 1.0.0
zaino login                     # must print a verification URL and a code
# approve in the browser, then:
zaino exec "reply with exactly: OK"
```

The last line is the only proof that matters: it exercises the device-issued
key, the responses shim, and streaming together. It has not yet been run with a
live key — see the end-to-end note in `PLAN-cli-release.md`.

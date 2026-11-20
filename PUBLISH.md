# Publishing Zaino to npm

`zaino 1.0.1` is published. This file records how it got there, what is still
open, and how the next release publishes without any manual step.

---

## Published names

The `@zaino` scope is not ours, so every package ships unscoped:

| Package | Purpose |
|---|---|
| `zainocli` | wrapper; resolves the platform binary through `optionalDependencies` aliases |
| `zainocli-linux-x64`, `-linux-arm64`, `-darwin-x64`, `-darwin-arm64` | platform binaries |
| `zainocli-win32-x64`, `-win32-arm64` | defined, but no Windows build is produced yet |
| `zainocli-sdk` | TypeScript SDK |
| `zainocli-responses-api-proxy` | responses API shim |

The four platform binaries publish under the **single registry name `zainocli`**,
each as a version (`1.0.1-linux-x64`) behind its own dist-tag (`linux-x64`). The
wrapper's aliases point at those dist-tags.

---

## How a release publishes

Tag and push — `.github/workflows/rust-release.yml` does the rest:

```bash
git tag rust-v1.0.1 && git push origin rust-v1.0.1
```

`publish-npm` authenticates with **OIDC trusted publishing**, not a stored
token. It grants `id-token: write`, strips the empty `_authToken` line that
`setup-node` writes for `registry-url`, and upgrades the npm CLI (OIDC needs
11.5.1+, and setting dist-tags over OIDC needs 11.21.0+).

For that to work, each package needs a trusted publisher on npmjs.com
(**Settings → Trusted Publisher**):

- Publisher: GitHub Actions
- Repository: `Louay0007/zaino-cli`
- Workflow filename: `rust-release.yml`
- Environment name: **blank** — the job declares no `environment:`

Once set, delete any `NPM_TOKEN` secret: OIDC needs none, and npm uses a
configured token *instead of* attempting the exchange.

### The first-publish rule

npm accepts a trusted publisher only for a package that already exists, and the
settings page does not exist until the first publish. So a brand-new package
needs one manual publish before CI can own it:

```bash
npm publish <tarball>.tgz --access public
```

`publish-npm` handles this gracefully: a name that has never been published is
collected and reported at the end (with these exact commands) while everything
else still ships. It treats only a definitive `E404` as "never published" — a
network or auth failure aborts, so a transient error can never silently skip a
live package.

---

## Ordering

Platform packages must exist before the wrapper, because the wrapper's
`optionalDependencies` aliases resolve to their dist-tags. The job already
publishes in that order, and it publishes serially: every platform tarball is a
new version of the same packument, so concurrent writes would race with HTTP 409.

---

## What is still missing

| Gap | Effect | Fix |
|---|---|---|
| macOS binaries unsigned | Gatekeeper warns on first run | Apple Developer Program + the `AKV_*` secrets |
| No Windows build | `zainocli-win32-*` never produced | enable the Windows build in the release matrix |
| No Homebrew tap | doctor's `brew upgrade --cask zaino` fails | publish a cask or drop the advice |
| Installers not hosted | `zaino app` has nothing to fetch | place the DMGs in `DOWNLOADS_DIR` on the VPS |

---

## Verify a publish actually works

On a clean machine, with no checkout:

```bash
npm install -g zainocli
zaino --version                 # must print: zaino 1.0.1
zaino login                     # must print a verification URL and a code
# approve in the browser, then:
zaino exec "reply with exactly: OK"
```

The last line is the only proof that matters: it exercises the device-issued
key, the responses shim, and streaming together.

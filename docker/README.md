# Docker build environments

Reproducible toolchains for the WebTerm desktop (GPUI) client. Every build in CI
goes through these images, so a CI failure is a code failure — never a
"worked on my machine" toolchain difference.

| File | Purpose | Extras on top of the Rust base |
| --- | --- | --- |
| `Dockerfile.build` | Builds both Linux AppImage variants | Go 1.27, pinned linuxdeploy + appimagetool, full window/GL/font headers |
| `Dockerfile.test` | `go test ./...` + the Rust workspace tests | Go 1.27, window/GL/font headers |

Both are **environment-only**: they never `COPY` the source. The repo is mounted
at `/src` at run time, which keeps the images cacheable across commits and lets
the repo own the build steps (`docker/build-appimage.sh` and
`desktop-gpui/scripts/build-appimage.sh`).

> Not to be confused with `../Dockerfile.build.web` + `../docker-compose.build.web.yaml`
> at the repo root: those package the **web** deployment (Go server + `fe/dist`
> into `compiled/web/`). This directory packages the **desktop** client.

## Build the AppImages

```bash
docker build -f docker/Dockerfile.build -t webterm-build .
docker run --rm -v "$PWD:/src" -w /src webterm-build bash docker/build-appimage.sh
# -> desktop-gpui/dist/WebTerm-GPUI_<version>_x86_64.AppImage       lean
#    desktop-gpui/dist/WebTerm-GPUI_<version>_x86_64-gtk.AppImage   GTK3 bundled
#    desktop-gpui/dist/SHA256SUMS.txt
```

Two variants exist because the client links `libgtk-3.so.0` unconditionally (see
`crates/webterm/src/gtk_theme.rs`), and the two audiences want different things:

| Variant | Ships | Needs GTK3 on the host | Size |
| --- | --- | --- | --- |
| `..._x86_64.AppImage` | `libxkbcommon.so.0`, `libxkbcommon-x11.so.0` | **yes** | ~22 MB |
| `..._x86_64-gtk.AppImage` | 56 libraries, including the whole GTK3 stack | no | ~31 MB |

Pick one explicitly when you do not need both:

```bash
docker run --rm -v "$PWD:/src" -w /src webterm-build \
  bash -c 'desktop-gpui/scripts/build-appimage.sh --gtk-bundled'
```

Useful mounts to keep caches warm between runs (all optional):

```bash
docker run --rm -v "$PWD:/src" -w /src \
  -v "$HOME/.cargo/registry:/usr/local/cargo/registry" \
  -v "$HOME/.cargo/git:/usr/local/cargo/git" \
  -v "$HOME/go/pkg/mod:/root/go/pkg/mod" \
  webterm-build bash docker/build-appimage.sh
```

Running as the host user instead of root keeps `desktop-gpui/target/` and
`desktop-gpui/dist/` writable for your own `cargo` afterwards:

```bash
docker run --rm --user "$(id -u):$(id -g)" -e HOME=/tmp/webterm-build \
  -v "$PWD:/src" -w /src webterm-build bash docker/build-appimage.sh
```

## Run the tests

```bash
docker build -f docker/Dockerfile.test -t webterm-test .

# 1. backend
docker run --rm -v "$PWD:/src" -w /src/be webterm-test go test ./...

# 2. the fixture the integration tests spawn (gitignored: absent on a fresh clone)
mkdir -p desktop-gpui/test-support
docker run --rm -v "$PWD:/src" -w /src/be webterm-test \
  go build -o ../desktop-gpui/test-support/backend ./cmd/server

# 3. the workspace, every live check on
docker run --rm -v "$PWD:/src" \
  -e TEST_BACKEND_PATH=/src/desktop-gpui/test-support/backend \
  -w /src/desktop-gpui webterm-test \
  cargo test --workspace --locked
```

`TEST_BACKEND_PATH` must be absolute: cargo runs each test binary from its own
crate directory, so the per-crate candidate lists only resolve for some of them —
without the variable 4 of the 7 live checks (supervisor `integration`,
`loopback_test`, `chaos_test`) skip themselves and still report `ok`. Step 3
without step 2 also works, in which case all 7 skip and the suite still passes;
`test-linux.yml` builds the fixture inside this same image on purpose, so the
live tests exercise a backend compiled by the same glibc as the test process.

## Why Debian bookworm

The AppImage must run on the maintainer's Fedora and on Ubuntu alike. Building
against bookworm's glibc 2.36 (older than both) means the produced binary only
ever references symbols every target already exports. libvulkan, the GPU drivers
and the X11/Wayland client libraries are always resolved from the host at run
time by SONAME.

The GTK3 stack is the one dependency that is a choice, and the two variants are
that choice made explicit:

* **lean** — `desktop-gpui/scripts/build-appimage.sh` tells `linuxdeploy` to skip
  `libgtk-3.so.0`, `libglib-2.0.so.0` and `libgobject-2.0.so.0` (the only
  GTK-family entries in the client's `DT_NEEDED`), then prunes `usr/lib` back to
  the libraries the client links directly. The prune is not optional:
  `--exclude-library` only filters *names* out of the copy, it does not stop
  `linuxdeploy` walking the closure, so without it ~53 GTK libraries (gdk, pango,
  atk, cairo, gio, …) still land in the bundle. The script fails the build if the
  result drifts from the contract.
* **GTK3 bundled** — no exclusions, so `linuxdeploy` ships the whole tree and the
  AppImage starts on a host with no GTK3 at all.

## How the AppImage tools get in

`desktop-gpui/scripts/build-appimage.sh` fetches linuxdeploy and appimagetool
itself, through `LINUXDEPLOY_URL` / `APPIMAGETOOL_URL`, and runs each one with
`--appimage-extract-and-run` (so no `/dev/fuse` is needed). `Dockerfile.build`
therefore downloads both AppImages into the image and exports those two env vars
as `file:///opt/...` URLs: the repo script is unchanged, no network is needed
during the build, and the tool versions are pinned by the image instead of by
"whatever `continuous` was that day". The image build itself runs each tool's
`--version` as a smoke test.

## What CI does with them

One workflow per responsibility, so nothing runs twice:

| workflow | responsibility | triggers |
| --- | --- | --- |
| `test-linux.yml` | `go test` + fixture + `cargo test --workspace` in the test image | `be/**`, `desktop-gpui/**`, `docker/**` on `master` + PRs |
| `desktop-windows.yml` | native Windows build/test (no GTK needed: `gtk`/`x11rb` are Linux-gated dependencies) | `be/**`, `desktop-gpui/**` on `master` + PRs |
| `build-appimage.yml` | both AppImage variants | every push/PR (so a `v*` tag always releases) |

Both variants are uploaded as one Actions artifact and published to the same
GitHub Release — a rolling **`latest`** on `master` pushes, a versioned release
for `v*` tags — which keeps permanent download URLs that survive merges:

```bash
gh release download latest --repo Aorus22/web-term --pattern '*.AppImage'
```

# Unlink

An mdBook backend that validates links in your book.

This is a backend for [mdBook](https://rust-lang.github.io/mdBook/). So, it does not work as a standalone CLI.

## Installation

Build from crates.io:

```bash
cargo install mdbook-unlink
```

Use prebuilt binaries:

```bash
# Replace UNLINK_VERSION with the version you want to install (e.g. 0.1.0)
curl -sSL https://github.com/ShaunSHamilton/mdbook-unlink/releases/download/v${UNLINK_VERSION}/x86_64-unknown-linux-gnu.tar.gz | tar -xz
```

## Configuration

Within your `book.toml` file, you can configure the following options:

```toml
[output.unlink]
ignore-files= ["String"]
# A list of glob patterns to ignore when checking links
ignore-links = ["String"]
# Whether or not to check draft chapters
# Default: true
check-drafts = true
# A list of files to include when checking links
include-files = ["String"]
```

## CI

Here is an example GitHub Actions workflow:

```yaml
name: Check Links

on:
  # Runs on PRs targeting main
  pull_request:
    branches: ["main"]
    # Only run on changes to the ./docs/ folder
    paths:
      - "docs/**"

  # Allows this workflow to be manually run from the Actions tab
  workflow_dispatch:

permissions:
  contents: read

jobs:
  build:
    runs-on: ubuntu-latest
    env:
      MDBOOK_VERSION: 0.4.28
      UNLINK_VERSION: 0.1.0
    steps:
      - uses: actions/checkout@8e5e7e5ab8b370d6c329ec480221332ada57f0ab # v3
      - name: Install mdBook
        run: |
          mkdir bin
          curl -sSL https://github.com/rust-lang/mdBook/releases/download/v${MDBOOK_VERSION}/mdbook-v${MDBOOK_VERSION}-x86_64-unknown-linux-gnu.tar.gz | tar -xz --directory=bin
      - name: Install mdbook-unlink
        run: |
          curl -sSL https://github.com/ShaunSHamilton/mdbook-unlink/releases/download/v${UNLINK_VERSION}/x86_64-unknown-linux-gnu.tar.gz | tar -xz --directory=bin
          echo "$PWD/bin" >> $GITHUB_PATH
      - name: Check Links
        run: cd docs && mdbook build
```

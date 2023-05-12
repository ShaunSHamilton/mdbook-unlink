# Unlink

A mdBook backend that validates links in your book.

## Configuration

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

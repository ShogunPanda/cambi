# cambi

[![Version](https://img.shields.io/crates/v/cambi.svg)](https://crates.io/crates/cambi)
[![Dependencies](https://img.shields.io/librariesio/release/cargo/cambi)](https://libraries.io/cargo/cambi)

`cambi` is a tool that infers semantic version bumps from conventional commits, updates CHANGELOG.md, and manages GitHub releases.

It can:

- compute the next semantic bump (`major` / `minor` / `patch`),
- generate and maintain `CHANGELOG.md`,
- publish GitHub releases from git tag history.

Project page: https://sw.cowtech.it/cambi

## Features

- Conventional commit bump inference (`feat` => minor, breaking => major, default patch)
- Config layering (flags > env > config file > defaults)
- Changelog generation with optional custom template
- Changelog rebuild mode from tag history
- Optional changelog auto-commit
- GitHub release create/update/rebuild flows
- Notes-only mode (`release --notes-only`) to print generated notes locally
- Commit filtering with configurable ignore regexes

## Installation

### Prebuilt binaries

Download one of the prebuilt binaries for your platform, then make it executable:

- [Linux](https://github.com/ShogunPanda/cambi/releases/latest/download/cambi-linux)
- [macOS (Intel)](https://github.com/ShogunPanda/cambi/releases/latest/download/cambi-macos-intel)
- [macOS (Apple Silicon)](https://github.com/ShogunPanda/cambi/releases/latest/download/cambi-macos-arm)
- [Windows](https://github.com/ShogunPanda/cambi/releases/latest/download/cambi.exe)

Example:

```sh
curl -L -o cambi https://github.com/ShogunPanda/cambi/releases/latest/download/cambi-linux
chmod a+x cambi
```

### From source (Cargo)

```sh
cargo install cambi
```

## Usage

```text
cambi [OPTIONS] <COMMAND>
```

Global options:

- `-c, --config <CONFIG>`: optional explicit config file path
- `-p, --tag-pattern <TAG_PATTERN>`: override the release tag matcher regex
- `-v, --verbose`: enable verbose output
- `-h, --help`: print help
- `-V, --version`: print version

### Commands

- `version` (`v`): print the current version
- `semver` (`s`): compute semantic version information
- `update` (`u`): update project version files from detected or explicit target
- `changelog` (`c`): update `CHANGELOG.md` with the next release section
- `release` (`r`): publish releases on GitHub from git history derived by tags

#### `version` (`v`)

Print current version from the latest matching tag (or from `-f, --from-tag`).

```sh
cambi version
cambi v
cambi version --from-tag v1.2.3
```

Options:

- `-f, --from-tag <FROM_TAG>`: override start tag instead of auto-detecting latest version tag
- `-c, --config <CONFIG>`
- `-p, --tag-pattern <TAG_PATTERN>`
- `-v, --verbose`
- `-h, --help`

#### `semver` (`s`)

Compute next bump type from commits since latest matching tag (or from `-f, --from-tag`).

```sh
cambi semver
cambi s
cambi semver --from-tag v1.2.3
```

Options:

- `-f, --from-tag <FROM_TAG>`: override start tag instead of auto-detecting latest version tag
- `-c, --config <CONFIG>`
- `-p, --tag-pattern <TAG_PATTERN>`
- `-v, --verbose`
- `-h, --help`

#### `update` (`u`)

Update project version files. By default it infers the bump from commits. You can override detection by passing:
- a bump: `major`, `minor`, `patch`
- an exact semver: `1.2.3` or `v1.2.3`

```sh
cambi update
cambi u
cambi update major
cambi update 1.4.0
cambi update --commit
cambi update --commit --commit-message "chore: bump app version"
cambi update --commit --tag
```

Options:

- `-f, --from-tag <FROM_TAG>`: override start tag instead of auto-detecting latest version tag
- `-o, --commit`: commit updated version file
- `-m, --commit-message <MESSAGE>`: custom commit message (requires `--commit`)
- `-t, --tag`: create a git tag for the updated version (requires `--commit`)
- `-c, --config <CONFIG>`
- `-p, --tag-pattern <TAG_PATTERN>`
- `-v, --verbose`
- `-h, --help`

Supported update targets (first match wins):

- `Cargo.toml`
- `package.json`
- `pyproject.toml`
- `*.gemspec`
- `mix.exs`
- `pubspec.yaml`
- `Package.swift`
- `version` / `VERSION`

#### `changelog` (`c`)

Update `CHANGELOG.md` with the next pending release section.

```sh
cambi changelog
cambi c --dry-run
cambi changelog --commit
cambi changelog --commit --commit-message "chore: update release notes"
cambi changelog --rebuild
```

Options:

- `-r, --rebuild`: regenerate `CHANGELOG.md` from the first commit
- `-o, --commit`: auto-commit if `CHANGELOG.md` is the only changed file
- `-m, --commit-message <MESSAGE>`: custom commit message (requires `--commit`)
- `-d, --dry-run`: preview changes without writing files
- `-c, --config <CONFIG>`
- `-p, --tag-pattern <TAG_PATTERN>`
- `-v, --verbose`
- `-h, --help`

#### `release` (`r`)

Create/update GitHub releases from git tags and commits.

```sh
cambi release
cambi r --dry-run
cambi release --rebuild
cambi release --owner my-org --repo my-repo --token "$GH_RELEASE_TOKEN"
cambi release 1.2.3 --prerelease
cambi release --notes-only
```

Options:

- `-r, --rebuild`: delete/recreate releases from scratch
- `-n, --notes-only`: print only the notes that would be used for the release body
- `-t, --token <TOKEN>`: override GitHub token
- `-o, --owner <OWNER>`: override GitHub owner/organization
- `-u, --repo <REPO>`: override GitHub repository
- `-d, --dry-run`: preview release actions without API calls
- `-a, --prerelease`: mark the GitHub release as a pre-release (requires positional target)
- `-c, --config <CONFIG>`
- `-p, --tag-pattern <TAG_PATTERN>`
- `-v, --verbose`
- `-h, --help`

Notes:

- release tags are `v`-prefixed (for example `v1.2.3`)
- release title omits `v` (for example `1.2.3`)
- `-n, --notes-only` conflicts with `--rebuild`, `--dry-run`, `--token`, `--owner`, `--repo`

## Configuration

### Config files

- Global: `~/.config/cambi.yml`
- Local (project): `./cambi.yml`
- Explicit: `--config` / `-c` `path/to/file.yml`

Local config overlays global config.

### Environment variables

- `CAMBI_TOKEN` (preferred) / `GH_RELEASE_TOKEN`
- `CAMBI_OWNER`
- `CAMBI_REPO`
- `CAMBI_TAG_PATTERN`
- `CAMBI_CHANGELOG_TEMPLATE`
- `CAMBI_IGNORE_PATTERNS` (semicolon-separated regex list)
- `CAMBI_VERBOSE` (`1`, `true`, `yes`)

### Defaults

- Tag pattern: `^v\d+\.\d+\.\d+$`
- Ignore patterns:
  - ^.+: fixup$
  - ^.+: wip$
  - ^fixup: .+$
  - ^wip: .+$
  - ^fixup$
  - ^wip$
  - ^Merge .+$

### Example `cambi.yml`

```yaml
token: ghp_xxx
owner: my-org
repo: my-repo
tag_pattern: '^v\d+\.\d+\.\d+$'
ignore_patterns:
  - "^docs: .+$"
  - "^chore: .+$"
changelog_template: |
  ### $DATE / $VERSION

  $COMMITS
```

Template placeholders:

- `$DATE`
- `$VERSION`
- `$COMMITS` (already bullet-formatted)

## Contributing

- Check open issues/PRs first
- Fork and create a feature branch
- Add or update tests for behavior changes
- Open a PR

## License

Copyright (C) 2026 and above Shogun (shogun@cowtech.it).

Licensed under the ISC license.

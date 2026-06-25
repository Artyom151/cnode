# cnpm

A Node.js package manager written in Rust. Compatible with the npm registry, package.json format, and package-lock.json.

## Overview

cnpm is a reimplementation of the npm package manager in Rust. It mirrors the npm CLI interface and uses the official npm registry API, making it a drop-in replacement for common npm workflows. The project includes three binaries: cnpm (package manager), cnpx (binary executor), and cnode (Node.js runtime info).

## Features

### Core package management
- Install packages from the npm registry with full dependency resolution
- Uninstall packages and update package.json
- Update individual or all packages to latest compatible versions
- Transitive dependency resolution for nested dependencies
- Semver specification support (^, ~, >=, x-ranges, exact versions)
- package-lock.json generation with integrity hashes

### Download and caching
- Concurrent package downloads with streaming progress indication
- Tarball integrity verification (SHA-256 and SHA-512 SRI format)
- Local on-disk caching with automatic reuse on subsequent installs
- Cache management commands (clean, verify, list)
- Configurable cache directory

### Registry operations
- Package metadata and version information retrieval
- Full-text package search via the npm search API
- Package information display (description, version history, dependencies)
- Security audit via npm advisory API
- Configurable registry URL (defaults to registry.npmjs.org)

### Project scaffolding
- Interactive project initialization (cnpm init)
- Scaffold new projects from scratch (cnpm new)
- npm script runner with argument passthrough (cnpm run)
- Binary execution from node_modules/.bin (cnpm exec / cnpx)

### Configuration
- TOML-based configuration file (~/.cnpmrc)
- Registry, cache directory, parallel downloads, and strict mode settings
- Command-line flag overrides for all config options

### Extensibility (optional)
- N-API bindings for native Node.js addon integration (feature-gated)

## Installation

### From source

```sh
git clone https://github.com/Artyom151/cnode
cd cnode
cargo build --release
```

The binaries are placed in `target/release/`:

```sh
cp target/release/cnpm ~/.local/bin/
cp target/release/cnpx ~/.local/bin/
cp target/release/cnode ~/.local/bin/
```

Or add the directory to your PATH.

### Requirements

- Rust 1.70 or later
- OpenSSL development libraries (for HTTPS support via reqwest)
- A C compiler (for native addon builds, optional)

### Windows

On Windows, cnpm works with both cmd.exe and PowerShell. The cnpx binary automatically handles .cmd and .ps1 shims in node_modules/.bin.

```sh
cargo build --release
copy target\release\cnpm.exe C:\tools\
copy target\release\cnpx.exe C:\tools\
```

## Usage

### Installing packages

```sh
# Install all dependencies from package.json
cnpm install

# Install specific packages
cnpm install express
cnpm install lodash@4.17.21
cnpm install typescript react@^18.0.0

# Install and save to package.json dependencies
cnpm install axios --save

# Install scoped packages
cnpm install @angular/core
cnpm install @angular/core@^16.0.0

# Install local package
cnpm install ./path/to/package.tgz
```

### Uninstalling packages

```sh
cnpm uninstall lodash

# Uninstall and update package.json
cnpm uninstall lodash --save
```

### Updating packages

```sh
# Update all packages to latest compatible versions
cnpm update --all

# Update a specific package
cnpm update express

# Update multiple packages
cnpm update lodash axios
```

### Searching the registry

```sh
# Search for packages matching a query
cnpm search react
cnpm search typescript
cnpm search "css framework"

# Limit results
cnpm search react --limit 5
```

### Getting package information

```sh
cnpm info express
cnpm info typescript
cnpm info @angular/core
```

### Listing installed packages

```sh
# List all installed packages
cnpm list

# Show with specific depth
cnpm list --depth 1

# List globally installed packages
cnpm list --global
```

### Creating a project

```sh
# Interactive initialization
cnpm init

# Scaffold a new project
cnpm new my-project
cd my-project
```

The new command creates:
- package.json with standard fields
- index.js entry point
- README.md project documentation
- src/ directory for source files

### Running scripts

```sh
# Run a script defined in package.json
cnpm run test
cnpm run build
cnpm run start

# Pass additional arguments to the script
cnpm run test -- --coverage
cnpm run build -- --production
```

### Executing binaries

```sh
# Both commands do the same thing
cnpm exec eslint src/
cnpx prettier --write src/
cnpx tsc --version
cnpx mocha tests/
```

The exec command looks for the binary in node_modules/.bin and runs it. On Windows it resolves .cmd and .ps1 shims automatically.

### Security audit

```sh
# Run a security audit against the npm advisory database
cnpm audit
```

Displays known vulnerabilities in installed packages.

### Cache management

```sh
# List cached tarballs
cnpm cache list

# Verify cache integrity
cnpm cache verify

# Clear the cache
cnpm cache clean
```

### Configuration

```sh
# List current configuration
cnpm config list

# Get a specific configuration value
cnpm config get registry
cnpm config get cache-dir
cnpm config get parallel-downloads

# Set a configuration value
cnpm config set registry https://registry.npmmirror.com
cnpm config set parallel-downloads 8
cnpm config set strict true

# Delete a configuration value
cnpm config delete registry
```

### Cleaning

```sh
# Clean the cache directory
cnpm clean
```

### Version information

```sh
cnpm version
```

## Configuration reference

cnpm reads configuration from `~/.cnpmrc` in TOML format:

```toml
registry = "https://registry.npmjs.org"
cache_dir = "~/.cache/cnpm"
parallel_downloads = 4
strict = false
```

### Options

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| registry | string | https://registry.npmjs.org | npm registry URL |
| cache_dir | string | ~/.cache/cnpm (Linux/macOS) or %LOCALAPPDATA%/cnpm (Windows) | Package tarball cache location |
| parallel_downloads | integer | 4 | Maximum concurrent downloads |
| strict | boolean | false | Enable strict mode |

### Command-line flags

Flags override both config file defaults:

| Flag | Overrides |
|------|-----------|
| --registry | registry |
| --cache-dir | cache_dir |
| --save-dev | (adds to devDependencies) |
| --save-optional | (adds to optionalDependencies) |

## Binary naming

| Binary | Purpose |
|--------|---------|
| **cnpm** | Package manager CLI (the primary tool) |
| **cnpx** | Execute node_modules binaries (npx-like functionality) |
| **cnode** | Node.js runtime information tool |

## Project structure

```
src/
  bin/
    cnpm.rs          Package manager CLI entry point
    cnpx.rs          Binary executor entry point
    cnode.rs         Node.js runtime info entry point
  cli/
    mod.rs           CLI argument parsing (clap derive)
    commands.rs      Module declarations for all commands
    commands/
      install.rs     Package installation with dependency resolution
      uninstall.rs   Package removal
      update.rs      Package updating
      search.rs      Registry search
      info.rs        Package metadata display
      list.rs        Installed package tree display
      init.rs        Interactive project initialization
      run.rs         npm script runner
      new.rs         Project scaffolding
      exec.rs        node_modules binary execution
      cache.rs       Cache management (clean, verify, list)
      config.rs      Configuration management (get, set, list, delete)
      clean.rs       Cache cleanup
      audit.rs       Security audit
      version.rs     Version display
  config.rs          TOML configuration file management
  downloader.rs      HTTP tarball download, integrity verification, extraction
  error.rs           Error type definitions
  lib.rs             Library root
  package.rs         Data models (Package, PackageVersion, DistInfo, etc.)
  registry.rs        npm registry HTTP API client
  resolver.rs        Dependency resolution
  bindings.rs        N-API bindings (optional, feature-gated)
```

## How it works

### Installation flow

1. Parse package specifications from command line or package.json
2. Fetch package metadata from the npm registry API
3. Resolve version constraints using semver logic
4. Recursively resolve all transitive dependencies
5. Fetch version information for every resolved package
6. Download package tarballs concurrently with progress display
7. Verify tarball integrity against SRI hashes
8. Extract tarballs into node_modules with correct directory structure
9. Generate package-lock.json with resolved versions and integrity hashes
10. Optionally update package.json with --save flag

### Version resolution

cnpm supports the following semver specifications:

| Spec | Example | Behavior |
|------|---------|----------|
| exact | 4.17.21 | Install exactly this version |
| caret (^) | ^4.0.0 | Compatible with minor version changes |
| tilde (~) | ~4.0.0 | Only patch-level changes |
| range | >=4.0.0 | Minimum version |
| wildcard | 4.x | Compatible with any minor version |
| latest | latest | Latest published version |

### Caching strategy

Downloaded tarballs are stored in the cache directory with the filename format `{name}-{version}.tar.gz`. On subsequent installs, cnpm checks the cache first and reuses cached tarballs without downloading again. Cache integrity can be verified with `cnpm cache verify`.

### Integrity verification

Each downloaded tarball is verified against the SRI hash provided by the npm registry. Both SHA-256 and SHA-512 formats are supported. The computed hash is stored in package-lock.json for reproducible builds.

## Comparison with npm

| Feature | npm | cnpm |
|---------|-----|------|
| Registry compatibility | Full | Full |
| package.json format | Full | Full |
| package-lock.json | v2/v3 | v2 |
| Install packages | Yes | Yes |
| Uninstall packages | Yes | Yes |
| Update packages | Yes | Yes |
| Search registry | Yes | Yes |
| Package info | Yes | Yes |
| List installed | Yes | Yes |
| Init project | Yes | Yes |
| Run scripts | Yes | Yes |
| npx-like exec | npx | cnpx |
| Security audit | Yes | Yes |
| Cache management | Yes | Yes |
| Offline mode | Partial | Yes (cached packages) |
| Concurrent downloads | Yes | Yes |
| Integrity check | SHA-512 | SHA-256 / SHA-512 |
| Package-lock integrity | Yes | Yes |
| N-API bindings | Built-in | Feature-gated |
| Binary size | ~50 MB (Node.js) | ~5 MB (standalone) |

## Performance

cnpm is designed to be fast:
- Written in Rust with zero-cost abstractions
- Async I/O with tokio for non-blocking network operations
- Concurrent downloads with configurable parallelism
- No Node.js runtime dependency (self-contained binary)
- Optimized release profile with LTO and single codegen unit

## Development

### Building

```sh
cargo build              # debug build
cargo build --release    # release build with optimizations
```

### Running tests

```sh
cargo test               # run all tests
cargo test -- --nocapture # show test output
cargo test install       # run tests matching name
```

### Code structure

- Each CLI command is a separate module under src/cli/commands/
- Core functionality is in separate modules (registry, downloader, resolver, config)
- The library root (lib.rs) re-exports public API
- Binary entry points are thin wrappers around library functions


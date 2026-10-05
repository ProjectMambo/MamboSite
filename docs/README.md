# MamboSite

<p align="left">
  <img src="https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/TypeScript-3178C6?style=flat-square&logo=typescript&logoColor=white" alt="TypeScript" />
  <img src="https://img.shields.io/badge/Next.js-000000?style=flat-square&logo=nextdotjs&logoColor=white" alt="Next.js" />
  <img src="https://img.shields.io/badge/GitHub_Pages-222222?style=flat-square&logo=githubpages&logoColor=white" alt="GitHub Pages" />
</p>
<p align="left">
  <img src="https://img.shields.io/badge/Maintenance-Active-brightgreen?style=flat-square" alt="Maintenance status: active" />
  <img src="https://img.shields.io/github/last-commit/ProjectMambo/MamboSite?style=flat-square&color=7a5fff" alt="Last commit" />
  <img src="https://img.shields.io/github/repo-size/ProjectMambo/MamboSite?style=flat-square&color=yellow" alt="Repository size" />
  <a href="../LICENSE"><img src="https://img.shields.io/github/license/ProjectMambo/MamboSite?style=flat-square&color=orange" alt="License" /></a>
</p>

MamboSite is a Markdown-first static site platform for Project Mambo. It reads repository-local Markdown, validates and compiles it with Rust, emits typed TypeScript data and theme CSS, renders it with MamboSite-owned React components, and exports static files for GitHub Pages.

MamboSite is authoring-tool agnostic. Project Mambo happens to maintain canonical documentation in an Obsidian vault and exports it with a separate `sync-docs` workflow; other users may maintain `docs/` directly or provide their own synchronization process.

## Motivation

Project Mambo sites need one predictable path from reviewable Markdown to a validated static artifact. MamboSite keeps content portable, catches route and reference errors before rendering, and centralizes the shared web runtime without tying authors to a particular editor.

### Start here

| Goal | First document |
|---|---|
| Read the canonical Wiki documentation | [projectmambo.org/mambosite/](https://projectmambo.org/mambosite/) |
| Create or expand Markdown pages | [Authoring guide](Authoring%20Guide.md) |
| Configure, build, or deploy a site | [Build and deployment](Build%20and%20Deployment.md) |
| Understand or extend MamboSite itself | [Architecture](Architecture.md) |

### Goals

- Keep Markdown as the source of truth without requiring a particular editor.
- Accept a predictable, self-contained `docs/` tree inside each consuming repository.
- Compose sites through explicit mounts without filesystem symlinks.
- Preserve normal CommonMark and GitHub Flavored Markdown behaviour.
- Support Obsidian links, embeds, callouts, block references, and selected extensions.
- Put visible page components in body directives rather than large frontmatter objects.
- Generate deterministic, strongly typed TypeScript data rather than one handwritten page module per Markdown file.
- Validate routes, note links, note embeds, and component directives before the web build starts.
- Produce a fully static Next.js export suitable for GitHub Pages.

### Pipeline

```text
repository docs/
    -> MamboSite Rust compiler
    -> generated TypeScript + compiled theme/content assets
    -> versioned React runtime + MamboColour API/MamboFont asset-backed default theme
    -> static web build
    -> GitHub Pages
```

The compiler, React rendering engine, default components, theme contract, and static-framework adapter are maintained together in MamboSite. A website repository owns only its content, `mambo.toml`, optional `mambo.theme.toml`, and optional typed component overrides.

### Documentation map

Author content:

- [Authoring guide](Authoring%20Guide.md) — end-to-end workflow and copy-ready page patterns.
- [Content model](Content%20Model.md) — file hierarchy, routes, mounts, and frontmatter.
- [Markdown and directives](Markdown%20and%20Directives.md) — syntax and component reference.
- [Theme and components](Theme%20and%20Components.md) — layouts, responsive behavior, tokens, and overrides.

Configure and operate a site:

- [Build and deployment](Build%20and%20Deployment.md) — commands, static export, and GitHub Pages.
- [Diagnostics and testing](Diagnostics%20and%20Testing.md) — validation and quality gates.
- [Documentation sync](Documentation%20Sync.md) — optional Project Mambo authoring workflow.

Understand and extend MamboSite:

- [Architecture](Architecture.md)
- [Parsing and resolution](Parsing%20and%20Resolution.md)
- [TypeScript output](TypeScript%20Output.md)
- [Roadmap](Roadmap.md)

## Status

The initial end-to-end platform is implemented. The Rust compiler discovers and validates repository-local content, parses Markdown and MamboSite directives, resolves note references and embeds, and emits typed TypeScript plus a compiled theme. Local packages provide the framework-neutral content runtime, modular React registry, MamboColour-backed tokens, bundled MamboFont faces, MamboFolio-inspired default components, static build timestamps and footer content, and a thin Next.js static-export adapter.

`mbsite check`, `build`, `init`, and `deploy` cover the repository lifecycle. The current milestone supports the MamboFolio and MamboWiki integrations, including validated content-asset publication. Fragment transclusion, tree/table collections, masonry/carousel galleries, and search remain planned.

## User stories

- As an author, I can keep content in Markdown and receive actionable errors for invalid routes, links, embeds, assets, or directives.
- As a site owner, I can compose canonical documentation at stable routes and export a self-contained static site.
- As a theme maintainer, I can update the pinned MamboColour API and reviewed MamboFont assets through explicit provider boundaries.
- As a framework maintainer, I can evolve the compiler, runtime, components, and adapter behind explicit version pins.

## Getting started

### Prerequisites

- [Node.js](https://nodejs.org/) 20 or later and npm.
- [Rust](https://www.rust-lang.org/tools/install) 1.95.0 or later.
- Python 3 only when using a site's optional static preview script.
- Git, plus the [GitHub CLI](https://cli.github.com/) when `mbsite deploy` needs to re-dispatch an existing commit or a maintainer publishes a source release.

Install this checkout and the development command wrapper:

```bash
git clone https://github.com/ProjectMambo/MamboSite.git
cd MamboSite
npm ci
npm run build:packages
./script/install.sh
```

The installer links `mbsite` and the compatibility alias `mambosite` into `/usr/local/bin`; both run the workspace CLI with the repository's pinned Rust toolchain. Set `MAMBOSITE_BIN_DIR` when a different command directory is preferred. The installer uses `sudo` only when the selected directory is not writable and refuses to replace a pre-existing non-symlink command target.

Until the first packages are published, a consuming site can use `file:../MamboSite/packages/...` dependencies. Keep MamboSite and the site repository as siblings, install both dependency trees, and rebuild the shared packages after changing MamboSite.

The Rust workspace fetches MamboColour Rust crate `0.2.0` at the exact revision declared in `Cargo.toml` and locked in `Cargo.lock`. MamboColour embeds its four CSV palettes in the provider crate, so ordinary Rust builds use its typed role and seeded-random APIs without installing another command or materializing provider values into MamboSite source files. For provider-managed accents, MamboSite mixes its `u64` build seed with each slot before passing a `u32` seed to the provider, preserving paired light/dark selection while using the full build-seed input.

Updating the bundled MamboFont files is a maintainer task. Install `mbfont` from exact provider revision `62f199e3bc49f921434ff0082947441dd0fde07c`, which emits the expected `MamboFont-<Style>_v0.2.4.woff2` files, then run `npm run sync:theme`. The current MamboFont pilot emits `MamboFontPilot-*` artifacts and cannot replace that pinned provider. Ordinary package, site, and CI builds consume the committed fonts and stylesheet without requiring MamboFont.

## Dependencies

The ecosystem manifests are the authoritative direct-package inventories: Rust requirements and the MamboColour Git pin live in `Cargo.toml` and `Cargo.lock`; Node requirements and peer ranges live in the root/package `package.json` files and `package-lock.json`. Transitive packages are recorded by the lockfiles rather than repeated here.

| Dependency | Classification | Purpose | Provider, version pin, or source | Scope | Update path |
|---|---|---|---|---|---|
| Rust toolchain | Tool | Compile, format, test, and lint the Rust workspace | Rust `1.95.0` with `rustfmt` and Clippy is pinned in `rust-toolchain.toml`; crates declare minimum Rust `1.85` | Build/test for the compiler, CLI, and Rust libraries | Update the toolchain and `rust-version` deliberately, then run all Cargo gates |
| Direct Rust crates | Packages | Provide CLI parsing, Markdown, serialization, YAML/TOML, diagnostics, Unicode normalization, and safe temporary output | Version ranges in the workspace/crate `Cargo.toml` files; exact resolution in `Cargo.lock` | Build/runtime for the Rust workspace | Update manifests and lockfile together; run format, workspace tests, and Clippy |
| [MamboColour](https://github.com/ProjectMambo/MamboColour) | Package and sibling repository | Supply stable UI roles and paired seeded accent selection to `mambosite-theme` | Rust crate `0.2.0`, repository revision `39f0b4e45ce3bb7be8a3ecda8081d7f77c6948e0`, pinned in `Cargo.toml` and `Cargo.lock` | Ordinary Rust builds; provider CSV is embedded in the compiled dependency, with no provider command or runtime file access | Change the manifest revision, refresh `Cargo.lock`, inspect provider compatibility, and run the theme plus full workspace tests |
| Node.js, npm, and direct JavaScript packages | Platform, tool, and packages | Build and test the TypeScript runtime, React registry, default theme, and Next.js adapter | Node.js `20` or later; direct versions/ranges in `package.json` files and exact development resolution in `package-lock.json` | Build/runtime for the npm workspace and consuming static sites | Update manifests and lockfile; run `npm run check:packages` and `npm run test:packages` |
| [MamboFont](https://github.com/ProjectMambo/MamboFont) `mbfont` | Tool and sibling repository | Regenerate the four bundled WOFF2 faces and `mambofont.css` | Repository revision `62f199e3bc49f921434ff0082947441dd0fde07c`; artifact contract `0.2.4` in `script/sync_mambofont.mjs` | Maintainer theme refresh only; ordinary builds use committed assets | Install that revision, run `npm run sync:theme` and `npm run sync:theme:check`, inspect font/CSS changes, then run package tests |
| Git | Tool | Clone/version source and implement guarded build/deploy repository operations | System Git from its official distribution; no project-specific minimum is currently declared | Development and `mbsite deploy` runtime | Update the host tool deliberately, then run installer, CLI, deploy dry-run, and repository tests |
| [GitHub CLI](https://cli.github.com/) | Tool | Re-dispatch an existing deployment commit and publish maintainer source releases | Official `gh` distribution; authenticated session required for those remote paths | Optional deploy dispatch and maintainer release | Update `gh`, then exercise `mbsite deploy --dry-run` locally before a guarded live dispatch |
| GitHub Actions and Pages | External services | Build, upload, and deploy a consuming site's static artifact | Repository-owned workflow; the default scaffold pins `actions/checkout@v4`, `actions/setup-node@v4`, `actions/upload-pages-artifact@v3`, and `actions/deploy-pages@v4` | Optional production deployment, not compiler semantics | Update `templates/default/.github/workflows/pages.yml`, scaffold tests, and deployment documentation together, then verify a protected deployment |
| [MamboDocs](https://github.com/ProjectMambo/MamboDocs) checker | Tool and sibling repository | Validate repository and synchronized documentation structure | Compatible sibling checkout at `../MamboDocs`; no repository revision is currently pinned | Maintainer documentation validation only | Synchronize canonical docs, update the coordinated sibling checkout, then run `../MamboDocs/script/check-repository.sh --strict .` |

Python is not a MamboSite package or compiler dependency; a consuming site needs Python 3 only if it chooses the optional static preview script.

## Usage

Create a scaffold in an empty directory:

```bash
mbsite init my-site
```

The scaffold keeps authored pages in `docs/`, site settings in `mambo.toml`, and design tokens in `mambo.theme.toml`. The serialized/default theme and generated scaffold omit both accent keys, which leaves paired card accents in provider mode under MamboColour's seeded selection; adding either key selects site-owned custom mode, where both non-empty arrays are required. The scaffold substitutes the creating compiler's version into its MamboSite package and source-tag pins; until the npm packages are published, point them at the sibling checkout described above before installing dependencies. See the [Authoring guide](Authoring%20Guide.md) for page patterns and the [Build and deployment guide](Build%20and%20Deployment.md) for the complete operating model.

### Command line

```bash
mbsite check
mbsite build
mbsite init my-site
mbsite deploy
```

`check` validates without writing output. `build` performs content compilation and the configured static web build. `init` creates a safe default site scaffold in an empty repository. `deploy` builds, pushes committed work, and starts the configured GitHub Pages workflow; `workflow_dispatch` allows the same commit to be deployed again when there is nothing new to push.

The React packages and generated schema are versioned separately. A site pins compatible `@mambosite/runtime`, `@mambosite/react`, `@mambosite/theme-default`, and `@mambosite/next` versions, then replaces only named registry entries when it needs custom presentation. The packages currently live in this workspace; publishing the first release remains deployment work.

### Maintainer font update

```bash
npm run sync:theme
npm run sync:theme:check
```

Both scripts concern only the pinned MamboFont assets. The first calls `mbfont` through `script/sync_mambofont.mjs` and refreshes the packaged WOFF2 files and stylesheet. The check regenerates into a temporary directory and fails when those committed assets are stale. MamboColour updates instead move through `Cargo.toml`, `Cargo.lock`, and the Rust theme tests.

### Typical site workflow

```bash
mbsite check
npm run dev
npm run build
npm run deploy
```

`npm run dev` regenerates content and theme output before starting Next.js. `npm run build` delegates to one complete `mbsite build`, including the configured static renderer, and produces the configured output directory.

## Documentation

The [Authoring guide](Authoring%20Guide.md) covers page patterns, [Build and deployment](Build%20and%20Deployment.md) covers site operation, [Diagnostics and testing](Diagnostics%20and%20Testing.md) defines the quality gates, and [Architecture](Architecture.md) explains the compiler and runtime boundary. The documentation map above links the complete guide set.

## Project structure

```text
crates/               Rust compiler, theme, and CLI workspaces
packages/             versioned runtime, React, theme, and Next.js packages
script/               command wrapper, installer, and MamboFont asset adapter
docs/                 synchronized public and maintainer documentation
templates/            files emitted by mbsite init
```

## Validation

Run the complete repository gate:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm run check:packages
npm run test:packages
./script/test_install.sh
../MamboDocs/script/check-repository.sh --strict .
git diff --check
```

Maintainers changing bundled MamboFont assets also run `npm run sync:theme:check` with the pinned MamboFont revision documented above. MamboColour changes are covered by the pinned Cargo dependency and Rust theme tests.

## Development

Keep Markdown semantics in the compiler and presentation behind typed runtime contracts. Add the smallest regression test that demonstrates a parser, resolver, writer, adapter, or component behavior change, and update the relevant canonical guide in `notes/Docs/Projects/MamboSite/`.

### Deployment

`mbsite deploy` requires a clean deployment branch and never creates a commit. It runs a complete local build, pushes committed work when the branch is ahead, and otherwise dispatches the configured GitHub Pages workflow for the current commit.

Run a local deployment check without fetching, pushing, or dispatching:

```bash
mbsite deploy --dry-run
```

Before the first deployment, set the repository's Pages source to **GitHub Actions**, commit the generated workflow, and match `site.url` and `site.base_path` in `mambo.toml` to the public URL. The complete one-time setup and CI contract live in [Build and deployment](Build%20and%20Deployment.md).

### Technology direction

- Rust for discovery, parsing, resolution, validation, and TypeScript generation.
- [Comrak](https://github.com/kivikakk/comrak) as the initial CommonMark/GFM AST parser.
- TypeScript and React for the rendering runtime and components.
- Next.js static export with `output: "export"` for the final site.
- GitHub Actions and GitHub Pages for deployment.

## License

Distributed under the MIT License. See **[LICENSE](../LICENSE)** for more information.

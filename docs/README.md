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

## Start here

| Goal | First document |
|---|---|
| Create or expand Markdown pages | [Authoring Guide](Authoring%20Guide.md) |
| Configure, build, or deploy a site | [Build and Deployment](Build%20and%20Deployment.md) |
| Understand or extend MamboSite itself | [Architecture](Architecture.md) |

## Goals

- Keep Markdown as the source of truth without requiring a particular editor.
- Accept a predictable, self-contained `docs/` tree inside each consuming repository.
- Compose sites through explicit mounts without filesystem symlinks.
- Preserve normal CommonMark and GitHub Flavored Markdown behaviour.
- Support Obsidian links, embeds, callouts, block references, and selected extensions.
- Put visible page components in body directives rather than large frontmatter objects.
- Generate deterministic, strongly typed TypeScript data rather than one handwritten page module per Markdown file.
- Validate routes, note links, note embeds, and component directives before the web build starts.
- Produce a fully static Next.js export suitable for GitHub Pages.

## Pipeline

```text
repository docs/
    -> MamboSite Rust compiler
    -> generated TypeScript + compiled theme/content assets
    -> versioned React runtime + selected theme
    -> static web build
    -> GitHub Pages
```

The compiler, React rendering engine, default components, theme contract, and static-framework adapter are maintained together in MamboSite. A website repository owns only its content, `mambo.toml`, `mambo.theme.toml`, and optional typed component overrides.

## Documentation map

Author content:

- [Authoring Guide](Authoring%20Guide.md) — end-to-end workflow and copy-ready page patterns.
- [Content Model](Content%20Model.md) — file hierarchy, routes, mounts, and frontmatter.
- [Markdown and Directives](Markdown%20and%20Directives.md) — syntax and component reference.
- [Theme and Components](Theme%20and%20Components.md) — layouts, responsive behavior, tokens, and overrides.

Operate a site:

- [Build and Deployment](Build%20and%20Deployment.md) — commands, static export, and GitHub Pages.
- [Diagnostics and Testing](Diagnostics%20and%20Testing.md) — validation and quality gates.
- [Documentation Sync](Documentation%20Sync.md) — optional Project Mambo authoring workflow.

Extend MamboSite:

- [Architecture](Architecture.md)
- [Parsing and Resolution](Parsing%20and%20Resolution.md)
- [TypeScript Output](TypeScript%20Output.md)
- [Roadmap](Roadmap.md)

## Status

The initial end-to-end platform is implemented. The Rust compiler discovers and validates repository-local content, parses Markdown and MamboSite directives, resolves note references and embeds, and emits deterministic TypeScript plus a compiled theme. Local packages provide the framework-neutral content runtime, modular React registry, MamboFolio-inspired default components, and a thin Next.js static-export adapter.

`mbsite check`, `build`, `init`, and `deploy` cover the repository lifecycle. The current milestone supports the MamboFolio and MamboWiki integrations, including validated content-asset publication. Fragment transclusion, tree/table collections, masonry/carousel galleries, and search remain planned.

## Local setup

### Prerequisites

- [Node.js](https://nodejs.org/) 20 or later and npm.
- [Rust](https://www.rust-lang.org/tools/install) 1.95.0 or later.
- Python 3 only when using a site's optional static preview script.
- Git, plus the [GitHub CLI](https://cli.github.com/) when `mbsite deploy` needs to re-dispatch an existing commit.

Install this checkout and the development command wrapper:

```bash
git clone https://github.com/ProjectMambo/MamboSite.git
cd MamboSite
npm ci
npm run build:packages
./script/install.sh
```

The wrapper links `mbsite` into `/usr/local/bin` and runs the workspace CLI with the repository's pinned Rust toolchain. Set `MAMBOSITE_BIN_DIR` when a different command directory is preferred; the installer uses `sudo` only when the selected directory is not writable.

Until the first packages are published, a consuming site can use `file:../MamboSite/packages/...` dependencies. Keep MamboSite and the site repository as siblings, install both dependency trees, and rebuild the shared packages after changing MamboSite.

## Using MamboSite

Create a scaffold in an empty directory:

```bash
mbsite init my-site
```

The scaffold keeps authored pages in `docs/`, site settings in `mambo.toml`, and design tokens in `mambo.theme.toml`. Its dependency versions target the first MamboSite release; until those packages are published, point them at the sibling checkout described above before installing dependencies. See the [Authoring Guide](Authoring%20Guide.md) for page patterns and the [Build and Deployment guide](Build%20and%20Deployment.md) for the complete operating model.

### Command line

```bash
mbsite check
mbsite build
mbsite init my-site
mbsite deploy
```

`check` validates without writing output. `build` performs content compilation and the configured static web build. `init` creates a safe default site scaffold in an empty repository. `deploy` builds, pushes committed work, and starts the configured GitHub Pages workflow; `workflow_dispatch` allows the same commit to be deployed again when there is nothing new to push.

The React packages and generated schema are versioned separately. A site pins compatible `@mambosite/runtime`, `@mambosite/react`, `@mambosite/theme-default`, and `@mambosite/next` versions, then replaces only named registry entries when it needs custom presentation. The packages currently live in this workspace; publishing the first release remains deployment work.

### Typical site workflow

```bash
mbsite check
npm run dev
npm run build
npm run deploy
```

`npm run dev` regenerates content and theme output before starting Next.js. `npm run build` delegates to one complete `mbsite build`, including the configured static renderer, and produces the configured output directory.

## Deployment

`mbsite deploy` requires a clean deployment branch and never creates a commit. It runs a complete local build, pushes committed work when the branch is ahead, and otherwise dispatches the configured GitHub Pages workflow for the current commit.

Run a local deployment check without fetching, pushing, or dispatching:

```bash
mbsite deploy --dry-run
```

Before the first deployment, set the repository's Pages source to **GitHub Actions**, commit the generated workflow, and match `site.url` and `site.base_path` in `mambo.toml` to the public URL. The complete one-time setup and CI contract live in [Build and Deployment](Build%20and%20Deployment.md).

## Technology direction

- Rust for discovery, parsing, resolution, validation, and TypeScript generation.
- [Comrak](https://github.com/kivikakk/comrak) as the initial CommonMark/GFM AST parser.
- TypeScript and React for the rendering runtime and components.
- Next.js static export with `output: "export"` for the final site.
- GitHub Actions and GitHub Pages for deployment.

## License

Distributed under the MIT License. See **[LICENSE](../LICENSE)** for more information.

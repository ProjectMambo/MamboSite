# MamboSite

MamboSite is a Markdown-first static site platform for Project Mambo. It reads repository-local Markdown, validates and compiles it with Rust, emits typed TypeScript data and theme CSS, renders it with MamboSite-owned React components, and exports static files for GitHub Pages.

MamboSite is authoring-tool agnostic. Project Mambo happens to maintain canonical documentation in an Obsidian vault and exports it with a separate `sync-docs` workflow; other users may maintain `docs/` directly or provide their own synchronization process.

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
    -> generated TypeScript + compiled theme assets
    -> versioned React runtime + selected theme
    -> static web build
    -> GitHub Pages
```

The compiler, React rendering engine, default components, theme contract, and static-framework adapter are maintained together in MamboSite. A website repository owns only its content, `mambo.toml`, `mambo.theme.toml`, and optional typed component overrides.

## Design documents

- [Architecture](docs/Architecture.md)
- [Content Model](docs/Content%20Model.md)
- [Documentation Sync](docs/Documentation%20Sync.md) — optional Project Mambo authoring workflow
- [Markdown and Directives](docs/Markdown%20and%20Directives.md)
- [Parsing and Resolution](docs/Parsing%20and%20Resolution.md)
- [TypeScript Output](docs/TypeScript%20Output.md)
- [Theme and Components](docs/Theme%20and%20Components.md)
- [Build and Deployment](docs/Build%20and%20Deployment.md)
- [Diagnostics and Testing](docs/Diagnostics%20and%20Testing.md)
- [Roadmap](docs/Roadmap.md)

## Status

The initial end-to-end platform is implemented. The Rust compiler discovers and validates repository-local content, parses Markdown and MamboSite directives, resolves note references and embeds, and emits deterministic TypeScript plus a compiled theme. Local packages provide the framework-neutral content runtime, modular React registry, MamboFolio-inspired default components, and a thin Next.js static-export adapter.

`mambosite check`, `build`, `init`, and `deploy` cover the repository lifecycle. The current milestone supports the MamboFolio content patterns used during migration. Content-asset copying, fragment transclusion, tree/table collections, masonry/carousel galleries, search, and MamboWiki integration remain planned.

## Command line

```bash
mambosite check
mambosite build
mambosite init my-site
mambosite deploy
```

`check` validates without writing output. `build` performs content compilation and the configured static web build. `init` creates a safe default site scaffold in an empty repository. `deploy` builds, pushes committed work, and starts the configured GitHub Pages workflow; `workflow_dispatch` allows the same commit to be deployed again when there is nothing new to push.

The React packages and generated schema are versioned separately. A site pins compatible `@mambosite/runtime`, `@mambosite/react`, `@mambosite/theme-default`, and `@mambosite/next` versions, then replaces only named registry entries when it needs custom presentation. The packages currently live in this workspace; publishing the first release remains deployment work.

Install the development command wrapper from this checkout once:

```bash
./script/install.sh
```

It links `mambosite` into `/usr/local/bin` and runs the workspace CLI with the repository's pinned Rust toolchain. Set `MAMBOSITE_BIN_DIR` when a different command directory is preferred; the installer uses `sudo` only when the selected directory is not writable.

## Technology direction

- Rust for discovery, parsing, resolution, validation, and TypeScript generation.
- [Comrak](https://github.com/kivikakk/comrak) as the initial CommonMark/GFM AST parser.
- TypeScript and React for the rendering runtime and components.
- Next.js static export with `output: "export"` for the final site.
- GitHub Actions and GitHub Pages for deployment.

## License

MamboSite is intended to use the MIT License, consistent with the other Project Mambo repositories.

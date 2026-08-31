---
title: MamboSite
description: Markdown-first static site compiler for Project Mambo websites.
listed: false
---

# MamboSite

MamboSite is a Markdown-first static site compiler for Project Mambo. It reads repository-local Markdown, validates and compiles it with Rust, emits typed TypeScript data, renders it through a reusable web runtime, and will export static files for GitHub Pages.

MamboSite is authoring-tool agnostic. Project Mambo happens to maintain canonical documentation in an Obsidian vault and exports it with a separate `sync-docs` workflow; other users may maintain `docs/` directly or provide their own synchronization process.

The current compiler checkpoint covers configuration, deterministic source discovery and mounts, YAML frontmatter, a renderer-neutral CommonMark/GFM tree, schema-1 directives, selected Obsidian-compatible syntax, page/reference/embed graphs, structured diagnostics, and atomic TypeScript generation. It intentionally stops before asset processing, structural transclusion/component lowering, and the React rendering runtime.

## Goals

- Keep Markdown as the source of truth without requiring a particular editor.
- Accept a predictable, self-contained `docs/` tree inside each consuming repository.
- Compose sites through explicit mounts without filesystem symlinks.
- Preserve normal CommonMark and GitHub Flavored Markdown behaviour.
- Support Obsidian links, embeds, callouts, block references, and selected extensions.
- Put visible page components in body directives rather than large frontmatter objects.
- Generate deterministic, strongly typed TypeScript data rather than one handwritten page module per Markdown file.
- Validate every route, link, embed, component, and asset before the web build starts.
- Produce a fully static Next.js export suitable for GitHub Pages.

## Planned pipeline

```text
repository docs/
    -> MamboSite Rust compiler
    -> generated TypeScript + copied assets
    -> TypeScript rendering runtime
    -> Next.js static export
    -> GitHub Pages
```

## Design documents

- [Architecture](Architecture.md)
- [Content Model](Content%20Model.md)
- [Documentation Sync](Documentation%20Sync.md) — optional Project Mambo authoring workflow
- [Markdown and Directives](Markdown%20and%20Directives.md)
- [Parsing and Resolution](Parsing%20and%20Resolution.md)
- [TypeScript Output](TypeScript%20Output.md)
- [Theme and Components](Theme%20and%20Components.md)
- [Build and Deployment](Build%20and%20Deployment.md)
- [Diagnostics and Testing](Diagnostics%20and%20Testing.md)
- [Roadmap](Roadmap.md)

## Current implementation

The implementation keeps compiler stages and the renderer contract separate:

- `mambosite-core`: configuration, discovery, frontmatter, Markdown AST and dialect lowering, directive validation, routes, reference/embed graphs, and diagnostics.
- `mambosite-codegen-ts`: deterministic manifest/page modules and guarded atomic output replacement.
- `mambosite-cli`: command-line input, diagnostic formatting, exit codes, and invoking generation.
- `packages/runtime`: framework-neutral TypeScript interfaces for generated sites, pages, metadata, directives, references, source spans, and exhaustive Markdown nodes.

The compiler currently supports:

- A repository-local `mambo.toml`, `docs/` content root, and configured `index.md` entry.
- Physical pages plus explicit entry-page mounts from reserved `_mounts/` storage.
- Exact UTF-8 `.md` discovery while excluding reserved paths, `_info.md`, and `README.md`.
- Typed frontmatter with scalar or array tags, custom `data`, and migration support for MamboFolio's `period`, `wikiUrl`, and `githubUrl` fields.
- CommonMark, GFM tables/autolinks/task lists/strikethrough, footnotes, math, alerts, highlighting, wikilinks, and container block directives through an isolated Comrak adapter.
- Leaf and container directive parsing plus schema-1 name, property, type, range, placement, and nesting validation with normalized explicit defaults.
- Selected Obsidian-compatible comments, note embeds, aliases, heading/block fragments, and block IDs without depending on a vault or Obsidian installation.
- An owned serializable AST with content-root-relative paths and frontmatter-adjusted line, column, and UTF-8 byte spans.
- Deterministic routes, heading IDs, titles, descriptions, child relationships, collision checks, and stable page IDs.
- Resolved Markdown/wikilink page references, safe external links, heading and block fragments, outgoing links, backlinks, embed instances, cycle detection, and maximum-depth validation.
- Human or JSON diagnostics.
- Generated `manifest.ts`, one module per page, and a static page index. Existing output is replaced only when it carries MamboSite's generated-output marker.

This is not version 0.1 yet. The next compiler work is local asset resolution/copying, structural embed expansion, semantic component-node lowering, production filtering, and derived navigation/search data. The TypeScript renderer, MamboFolio-inspired components, Next.js shell, and GitHub Pages workflow follow that compiler contract.

## Running the current checkpoint

The repository includes a `mambo.toml` that compiles its own design documentation:

```bash
cargo run -p mambosite-cli -- check
cargo run -p mambosite-cli -- build
```

`check` validates without writing output. `build` writes the configured TypeScript tree, currently `.mambosite/generated/` for this repository. Pass `--config path/to/mambo.toml` for another site or `--diagnostics json` for machine-readable diagnostics.

Quality checks:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The current MamboFolio Markdown corpus is also used as a compatibility input during development; all 17 current pages parse and generate without modifying the MamboFolio repository. This covers content compatibility, not visual parity or preservation of its current singular `/project/...` URLs.

## Technology

- Rust for discovery, parsing, resolution, validation, and TypeScript generation.
- [Comrak](https://github.com/kivikakk/comrak) behind a MamboSite-owned CommonMark/GFM AST boundary.
- TypeScript and React for the rendering runtime and components.
- Next.js static export with `output: "export"` for the final site.
- GitHub Actions and GitHub Pages for deployment.

## License

MamboSite is intended to use the MIT License, consistent with the other Project Mambo repositories.

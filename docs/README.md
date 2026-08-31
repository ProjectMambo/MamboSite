
# MamboSite

MamboSite is a planned Markdown-first static site compiler for Project Mambo. It reads repository-local Markdown, validates and compiles it with Rust, emits typed TypeScript data, renders it through a reusable web runtime, and exports static files for GitHub Pages.

MamboSite is authoring-tool agnostic. Project Mambo happens to maintain canonical documentation in an Obsidian vault and exports it with a separate `sync-docs` workflow; other users may maintain `docs/` directly or provide their own synchronization process.

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

## Status

The project is in specification and prototyping. Commands, packages, schemas, and generated files described here are contracts for the first implementation; they are not yet available.

## Proposed command line

```bash
mambosite check
mambosite build
mambosite inspect /mambodot/commands
mambosite watch
```

`check` validates without writing generated output. `build` performs a deterministic full compilation. `inspect` explains how one page, route, or link was resolved. `watch` is a later development convenience and is not required for the first release.

## Technology direction

- Rust for discovery, parsing, resolution, validation, and TypeScript generation.
- [Comrak](https://github.com/kivikakk/comrak) as the initial CommonMark/GFM AST parser.
- TypeScript and React for the rendering runtime and components.
- Next.js static export with `output: "export"` for the final site.
- GitHub Actions and GitHub Pages for deployment.

## License

MamboSite is intended to use the MIT License, consistent with the other Project Mambo repositories.

---
description: System boundaries and repository structure for MamboSite.
title: Architecture
order: 10
---

# Architecture

## Status and terminology

This is the initial architecture contract for MamboSite. The words **must**, **should**, and **may** describe required behaviour, recommended behaviour, and optional behaviour respectively.

- **Authoring source**: wherever an author maintains original Markdown; outside MamboSite's contract.
- **Content root**: the self-contained repository directory MamboSite compiles, normally `docs/`.
- **Canonical content**: the one authored copy of a document before any optional synchronization.
- **Site entry**: the configured `index.md` that represents `/` inside a repository's content root.
- **Mount**: a mapping from another directory inside the content root into the site's route tree.
- **Synchronized content**: repository-local files produced by an optional external authoring workflow.
- **Compiler**: the Rust portion of MamboSite.
- **Runtime**: the TypeScript package that renders generated content nodes.
- **Web shell**: the site-specific Next.js application, theme, components, and configuration.

## Implementation checkpoint

The current compiler reaches deterministic TypeScript generation for parsed pages. It implements discovery and mounts, frontmatter/routes, an owned CommonMark/GFM AST, schema-1 directive parsing and validation, Obsidian-style comments/note embeds/block IDs, Markdown-link and wikilink resolution, aliases/fragments, outgoing links/backlinks, and embed graph cycle/depth validation.

Resolved directives and embeds have not yet been lowered into final renderer/component or structurally transcluded nodes. Local asset processing and copying, production filtering of draft pages, navigation/search derivation, React/Next.js rendering, static export, and GitHub Pages deployment remain outside this implemented slice. “Obsidian-compatible” therefore refers only to the explicitly supported syntax, not full Obsidian or plugin compatibility.

## Architectural boundary

MamboSite should be a compiler, not a content management system and not a second Markdown editor. It accepts content plus configuration and produces deterministic web data.

```text
Authoring                Compilation                          Presentation

repository docs/         Rust compiler                       Next.js web shell
----------------         -------------------------------     -----------------
Markdown          --->   discover and parse            --->  typed renderer
local assets             resolve links and mounts            site components
site entry               validate graph                      theme and CSS
                         emit TypeScript and assets           static HTML export
```

Rust owns meaning: routes, metadata, Markdown semantics, directives, links, embeds, assets, navigation, and diagnostics. TypeScript owns presentation: React elements, layouts, styling, client interactions, and static page rendering.

The boundary must remain data-oriented. Rust must not generate React page source for every document, and React must not reparse Markdown.

## MamboSite repository structure

The implementation should use a Cargo workspace with a small number of crates at first:

```text
MamboSite/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── README.md                     # project README
├── docs/                         # MamboSite documentation
├── crates/
│   ├── mambosite-core/
│   │   └── src/
│   │       ├── ast.rs
│   │       ├── compiler.rs
│   │       ├── config.rs
│   │       ├── diagnostic.rs
│   │       ├── dialect.rs
│   │       ├── directive.rs
│   │       ├── directive_registry.rs
│   │       ├── source.rs
│   │       ├── frontmatter.rs
│   │       ├── markdown.rs
│   │       ├── model.rs
│   │       ├── reference.rs
│   │       ├── route.rs
│   │       └── lib.rs
│   ├── mambosite-codegen-ts/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── serializer.rs
│   │       └── writer.rs
│   └── mambosite-cli/
│       └── src/main.rs
├── packages/
│   └── runtime/
│       ├── package.json
│       ├── tsconfig.json
│       └── src/
│           ├── index.ts
│           ├── directive.ts
│           ├── json.ts
│           ├── markdown.ts
│           ├── metadata.ts
│           ├── page.ts
│           ├── reference.ts
│           ├── site.ts
│           └── source.ts
├── schemas/
│   ├── content.schema.json
│   └── manifest.schema.json
├── examples/
│   ├── folio/
│   └── wiki/
├── tests/
│   └── fixtures/
│       ├── valid/
│       └── invalid/
```

Responsibilities:

- `mambosite-core` owns the compiler pipeline and all semantic models.
- `mambosite-codegen-ts` converts the validated intermediate representation into deterministic TypeScript modules.
- `mambosite-cli` handles commands, paths, terminal output, exit codes, and watch mode.
- `packages/runtime` currently defines the framework-neutral TypeScript contract. Renderer registries and React components will be added as separate presentation modules rather than mixed into these interfaces.
- `schemas` records versioned language-independent contracts for tooling and fixtures.
- `examples` demonstrates the two initial site shapes without becoming production source.
- `tests/fixtures` stores complete small content roots and expected diagnostics/output.

Do not split every core module into a crate initially. A new crate is justified only when it has a stable public boundary or independent consumers.

## Website repository structure

MamboFolio and MamboWiki should remain independent website repositories that consume MamboSite. Their compiler-facing structure is repository-local and independent of the author's editor or upstream storage layout:

```text
MamboWiki/
├── mambo.toml
├── README.md                     # the MamboWiki project's README
├── docs/
│   ├── index.md                  # site entry
│   ├── about.md                  # optional site-owned pages
│   └── _mounts/                  # materialized mounted sources
│       ├── mambocolour/
│       ├── mambodot/
│       ├── mambofinance/
│       ├── mambofolio/
│       ├── mambofont/
│       ├── mambosite/
│       └── mambowiki/
├── public/
│   └── mambo/                    # generated/copied assets
├── src/
│   ├── app/
│   │   └── [[...slug]]/page.tsx
│   ├── components/
│   ├── theme/
│   └── generated/mambo/          # generated TypeScript
├── next.config.ts
└── package.json
```

`docs/_mounts/` is excluded from ordinary route discovery and entered only through explicit mounts in `docs/index.md`. The repository therefore contains all compilation inputs without making storage paths into accidental routes.

MamboSite starts at this repository boundary; it does not create `docs/` from an editor, vault, database, or remote service. Project Mambo's separate synchronization workflow is specified in [[Documentation Sync]]. Other users can edit `docs/` directly or prepare the same structure with any external process.

Generated TypeScript and copied assets should not be considered authored files. Whether they are committed is a repository policy, but the preferred default is to rebuild them in CI and exclude them from version control.

## Compiler stages

The compiler runs ordered stages with an explicit intermediate representation between them:

1. Load and validate `mambo.toml`.
2. Discover the site entry and candidate files.
3. Read UTF-8 source and split frontmatter from body.
4. Parse frontmatter into typed core fields and custom data.
5. Parse Markdown into an AST with source spans.
6. Lower supported Obsidian-compatible syntax and MamboSite directives into compiler nodes.
7. Construct the source-file, mount, route, link, embed, and asset graphs.
8. Resolve routes, aliases, fragments, block identifiers, and assets.
9. Expand or retain embed nodes according to their mode.
10. Derive headings, descriptions, children, backlinks, navigation, and search text.
11. Validate all invariants and stop on errors.
12. Emit TypeScript and copy assets atomically.

The working compiler path covers configuration/discovery, frontmatter, the owned Markdown AST, and the supported directive/Obsidian lowering in stages 1–6. The page/route/mount portions of stage 7, non-asset reference work in stage 8, embed graph validation, heading/description/children/backlink derivation, validation, and deterministic TypeScript writing are also implemented. Structural embed expansion or component lowering in stage 9, asset indexing/resolution/copying, navigation/search derivation, production draft filtering, and the presentation/deployment pipeline remain.

A later stage must never silently repair an ambiguous earlier stage. For example, an ambiguous wikilink is an error, not a request to choose the first matching file.

## Configuration

Website-level settings belong in `mambo.toml`, not page frontmatter:

```toml
schema = 1
content_root = "docs"
entry = "index.md"
typescript_out = "src/generated/mambo"
assets_out = "public/mambo"

[site]
url = "https://projectmambo.org"
base_path = ""
trailing_slash = true
language = "en-SG"

[markdown]
raw_html = false
strict_links = true
max_embed_depth = 16
```

Paths in configuration are relative to the configuration file unless documented otherwise. Absolute paths must never be written into generated output.

## Major decisions

### Comrak as the initial Markdown engine

Comrak is the preferred first parser because it exposes an AST, supports CommonMark and GFM, carries source positions, and already offers extensions for frontmatter, wikilinks, alerts, and container block directives. MamboSite still owns its dialect: Obsidian embeds, aliases, block references, directive attributes, and semantic validation require a lowering layer around Comrak.

The parser adapter must be isolated behind MamboSite's own AST so switching parser libraries does not change the TypeScript contract.

### One compiler-owned intermediate representation

Neither Comrak nodes nor TypeScript rendering types should leak across the whole Rust codebase. The core must lower parsed input into owned MamboSite nodes with normalized paths and source spans. Resolution and code generation operate on these nodes.

### Static-only presentation

The first release targets static hosting. It must not depend on cookies, server actions, request-time route handlers, ISR, runtime filesystem access, or a Node.js server.

### Deterministic full builds first

The first implementation should perform a complete build on every invocation. Watch mode and incremental caching come only after deterministic full builds and dependency tracking are proven correct.

### Safe defaults

Raw HTML and arbitrary JavaScript expressions are disabled. Paths must remain inside configured roots. Current content traversal rejects symlinks; the future asset traversal must apply the same rule. External links are not fetched during a normal build.

## Dependency direction

```text
mambosite-cli
    -> mambosite-core
    -> mambosite-codegen-ts

mambosite-codegen-ts
    -> stable serialized site contract

TypeScript runtime
    <-> generated schema contract
    -> no Rust dependency
```

Core parsing and resolution must be usable as a library without invoking the CLI or writing files. This keeps unit tests fast and permits future editor tooling.

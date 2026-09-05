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
- **Runtime**: the versioned TypeScript and React packages that resolve and render generated content nodes.
- **Theme preset**: a versioned collection of default components and base styles.
- **Web adapter**: the framework-specific package that turns a compiled site into static routes and metadata.
- **Site customization**: repository-local theme settings and typed component overrides.

## Architectural boundary

MamboSite should be a compiler, not a content management system and not a second Markdown editor. It accepts content plus configuration and produces deterministic web data.

```text
Authoring                Compilation                         Presentation

repository docs/         Rust compiler                       MamboSite runtime
----------------         -------------------------------     -----------------
Markdown          --->   discover and parse            --->  React renderer
local assets             resolve links and mounts            component registry
site entry               validate graph                      selected theme
theme settings           emit TypeScript and theme CSS        web adapter/export
```

Rust owns compiled meaning: routes, metadata, Markdown semantics, directives, note links, note embeds, child relationships, backlinks, theme validation, and diagnostics. TypeScript owns presentation: graph queries, React elements, layouts, styling, client interactions, and static page rendering. MamboSite owns both sides of this boundary; consuming sites configure them and may override typed registry entries.

Content-asset resolution and publication are Rust responsibilities. A separate navigation model and search records are not implemented yet; the runtime must not guess those missing compiler semantics.

The boundary must remain data-oriented. Rust must not generate React page source for every document, and React must not reparse Markdown. Framework adapters must consume the same compiled data and component registry rather than inventing another content model.

## Runtime package boundary

The presentation implementation is split into independently versioned packages:

```text
@mambosite/runtime        schema contracts, content store, graph queries
@mambosite/react          renderer engine and typed component registry
@mambosite/theme-default  default components, shell, and styles
@mambosite/next           static Next.js route and metadata adapter
```

The default theme is not compiled into the Markdown language. A site may replace a theme package or override individual registry entries while retaining the same compiler and content. Its checked-in colour model is generated through MamboColour's public CLI, and its package bundles MamboFont web assets generated through MamboFont's public CLI. MamboFolio and MamboWiki consume that self-contained default package without invoking either provider during ordinary builds.

Compatibility has three explicit versions:

- `schemaVersion` identifies generated compiler data.
- Runtime and adapter packages use semantic versions.
- Theme packages use semantic versions independently of the compiler.

Each site pins compatible package versions in its lockfile. The current runtime accepts generated schema 1 and fails before rendering another schema. The four local packages move together as one workspace compatibility unit; GitHub source tags and npm package publication remain separate release events.

## MamboSite repository structure

The implementation uses one Cargo workspace and one npm workspace:

```text
MamboSite/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── README.md                     # project README
├── docs/                         # MamboSite documentation
├── crates/
│   ├── mambosite-core/
│   │   └── src/                 # parsing, resolution, IR, diagnostics
│   ├── mambosite-codegen-ts/
│   │   └── src/                 # TS serialization and managed writers
│   ├── mambosite-theme/
│   │   └── src/                 # typed settings, CSS and TS generation
│   └── mambosite-cli/
│       └── src/commands/        # check, build, init, deploy
├── packages/
│   ├── runtime/                 # generated contracts and content store
│   ├── react/                   # renderer and typed registry
│   ├── theme-default/           # default components, CSS, and font assets
│   └── next/                    # static Next.js adapter
├── script/
│   ├── sync_mambocolour.mjs     # refresh the checked-in colour model
│   └── sync_mambofont.mjs       # refresh bundled web fonts and CSS
└── templates/default/           # scaffold embedded by `mbsite init`
```

Responsibilities:

- `mambosite-core` owns the compiler pipeline and all semantic models.
- `mambosite-codegen-ts` converts the validated intermediate representation into deterministic TypeScript modules.
- `mambosite-theme` validates a complete settings model and compiles CSS and TypeScript metadata deterministically for a supplied accent seed.
- `mambosite-cli` handles lifecycle commands, safe paths, subprocess boundaries, terminal output, and exit codes.
- `packages/runtime` defines the generated contract and immutable graph/query API.
- `packages/react` renders normalized nodes through a complete typed registry.
- `packages/theme-default` supplies the replaceable MamboFolio-inspired presentation and bundled MamboFont assets.
- `packages/next` contains only Next-specific navigation, base-path, route, and metadata behavior.
- `script/sync_mambocolour.mjs` and `script/sync_mambofont.mjs` are maintainer-only consumer adapters for the providers' installed CLIs. They update reviewed, checked-in inputs; normal package and site builds remain self-contained.
- `templates/default` is the allowlisted scaffold embedded into the CLI.

Do not split every core module into a crate initially. A new crate is justified only when it has a stable public boundary or independent consumers.

## Website repository structure

MamboFolio and MamboWiki should remain independent website repositories that consume MamboSite. Their compiler-facing structure is repository-local and independent of the author's editor or upstream storage layout:

```text
MamboWiki/
├── mambo.toml
├── mambo.theme.toml              # optional site-specific overrides
├── README.md                     # the MamboWiki project's README
├── docs/
│   ├── index.md                  # site entry
│   ├── about.md                  # optional site-owned pages
│   ├── _assets/                  # non-routable content assets
│   └── _mounts/                  # materialized mounted sources
│       ├── mambocolour/
│       ├── mambodocs/
│       ├── mambodot/
│       ├── mambofinance/
│       ├── mambofolio/
│       ├── mambofont/
│       ├── mambosite/
│       └── mambowiki/
├── public/
│   └── mambo/                    # generated theme.css + content assets
├── src/
│   ├── app/
│   │   ├── page.tsx
│   │   └── [...slug]/page.tsx
│   ├── mambo/runtime.ts         # registry composition and runtime creation
│   └── generated/mambo/          # generated TypeScript
├── next.config.ts
└── package.json
```

`docs/_mounts/` is excluded from ordinary route discovery and entered only through explicit mounts in `docs/index.md`. The repository therefore contains all compilation inputs without making storage paths into accidental routes.

MamboSite starts at this repository boundary; it does not create `docs/` from an editor, vault, database, or remote service. Project Mambo's separate synchronization workflow is specified in [[Documentation Sync]]. Other users can edit `docs/` directly or prepare the same structure with any external process.

Generated TypeScript and theme CSS are not authored files. Whether they are committed is a repository policy, but the preferred default is to rebuild them in CI and exclude them from version control.

## Compiler stages

The current compiler runs these ordered stages:

1. Load and validate `mambo.toml`.
2. Read the configured entry frontmatter to obtain mounts.
3. Discover ordinary and mounted Markdown sources.
4. Read UTF-8 source and parse frontmatter into typed fields plus compatibility data.
5. Parse Markdown with Comrak into an owned AST with source spans.
6. Lower MamboSite directives, Obsidian comments, embeds, and block identifiers.
7. Validate directives, headings, blocks, routes, and mount namespaces.
8. Derive titles, descriptions, direct children, and stable page IDs.
9. Resolve Markdown links, wikilinks, fragments, note embeds, backlinks, embed cycles, and depth limits.
10. Validate `_assets/`, rewrite explicit `assets/...` references, and collect files for publication.
11. Stop on errors; otherwise emit managed TypeScript plus the managed theme/content-asset tree.

Navigation/search output and structural fragment transclusion remain later stages.

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

`site.base_path` is either empty or a canonical URL path with one leading slash, no trailing slash, and URL-safe segments. `assets_out` must be a non-empty URL-safe subdirectory of the repository's `public/` directory because its relative path becomes the public prefix for `theme.css` and compiled content assets.

## Major decisions

### Comrak as the initial Markdown engine

Comrak is the current parser because it exposes an AST, supports CommonMark and GFM, carries source positions, and offers extensions for wikilinks, alerts, and block directives. MamboSite still owns its dialect: Obsidian embeds, comments, block references, directive attributes, and semantic validation use a lowering layer around Comrak.

The parser adapter is isolated behind MamboSite's own AST so parser-library types do not become the TypeScript contract.

### One compiler-owned intermediate representation

Neither Comrak nodes nor TypeScript rendering types leak across the Rust workspace. Core lowers parsed input into owned MamboSite nodes with normalized paths and source spans; resolution and code generation operate on those nodes.

### Static-only presentation

The first release targets static hosting. It must not depend on cookies, server actions, request-time route handlers, ISR, runtime filesystem access, or a Node.js server.

### Complete builds first

The current implementation performs a complete build on every invocation. Semantic content, routes, TypeScript modules, and copied assets remain deterministic. The CLI deliberately gives collection accents a fresh build seed; `SOURCE_DATE_EPOCH` fixes that seed when byte-reproducible theme CSS is required. Watch mode and incremental caching come only after complete builds and dependency tracking are proven correct.

### Safe defaults

Raw HTML is preserved as text nodes and the default renderer does not inject it. Arbitrary JavaScript expressions are not an authoring feature. Configured paths must remain inside the repository, content-tree symlinks are rejected, and external links are not fetched.

## Dependency direction

```text
mambosite-cli
    +-> mambosite-core
    +-> mambosite-codegen-ts
    `-> mambosite-theme

mambosite-codegen-ts
    -> serializable validated site data (no core crate dependency)

@mambosite/react -> @mambosite/runtime
@mambosite/theme-default -> @mambosite/react + @mambosite/runtime
@mambosite/next -> @mambosite/react + @mambosite/runtime
```

Core parsing and resolution must be usable as a library without invoking the CLI or writing files. This keeps unit tests fast and permits future editor tooling.

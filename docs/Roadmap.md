---
title: Roadmap
description: Planned implementation phases, release criteria, non-goals, and open decisions.
order: 80
---

# Roadmap

## Current implementation status

The repository has passed the compiler-skeleton milestone and now contains an initial end-to-end implementation:

| Area | Status |
|---|---|
| Content discovery, core frontmatter types, routes, diagnostics | Implemented for schema 1; date/taxonomy validation remains planned |
| CommonMark/GFM and supported Obsidian-compatible AST | Implemented with source spans and fixtures |
| Directive parsing and registry validation | Implemented for the documented core registry |
| Page, link, embed, backlink, mount, and route graph | Implemented for note references; directive target edges, fragment transclusion, and content assets remain incomplete |
| TypeScript output | Manifest, page modules, index, and theme metadata are deterministic and atomically published; navigation, search, build-info, and copied content assets remain planned |
| React runtime | Versioned runtime, React registry, default theme, and Next adapter are implemented for current MamboFolio content |
| Theme settings | Rust-validated `mambo.theme.toml` compiles to generated CSS variables, responsive rules, and typed metadata |
| Lifecycle commands | `check`, full or content-only `build`, safe `init`, and guarded GitHub Pages `deploy` are implemented |
| Site migrations | MamboFolio's thin integration builds and exports successfully; MamboWiki and clean-CI deployment remain acceptance work |

The phases below describe the full 0.1 target. A phase is not considered complete merely because its first usable slice exists.

## Phase 0 — specification

- Agree on repository content-root and site hierarchy.
- Agree on frontmatter and route rules.
- Freeze directive grammar for schema version 1.
- Freeze the supported wikilink and embed-extension semantics.
- Define the Rust intermediate representation.
- Define the TypeScript runtime schema.
- Build representative valid and invalid fixture content before production code.

Deliverable: these design documents reviewed against repository-local MamboFolio and MamboWiki fixtures.

## Phase 1 — compiler skeleton

- Create Cargo workspace and CLI.
- Load `mambo.toml`.
- Discover and normalize source files.
- Parse and validate frontmatter.
- Implement logical paths, route derivation, exclusions, and source spans.
- Emit human and JSON diagnostics.

Deliverable: `mbsite check` understands files, metadata, and route conflicts without rendering Markdown.

## Phase 2 — Markdown and directives

- Integrate Comrak behind an adapter.
- Lower CommonMark/GFM nodes into the owned MamboSite AST.
- Implement leaf and container directive parsing.
- Validate core directive properties and contexts.
- Implement headings, fragments, callouts, comments, footnotes, math, and block IDs.

Deliverable: golden AST fixtures for every supported syntax feature.

## Phase 3 — graph and reference resolution

- Build page, route, mount, link, embed, and asset indexes.
- Resolve wikilinks, aliases, standard Markdown links, headings, and blocks.
- Implement mount semantics and final site routes.
- Implement embed modes, heading shifting, provenance, and cycle detection.
- Resolve and hash local assets.
- Derive children, navigation, backlinks, and related-content inputs.

Deliverable: `mbsite inspect` explains complete resolution for repository-local Wiki-shaped fixtures.

## Phase 4 — TypeScript generation

- Implement schema-versioned runtime types.
- Emit manifest, page modules, navigation, and build information.
- Implement deterministic and atomic writers.
- Copy and deduplicate referenced assets.
- Type-check golden output against the runtime package.

Deliverable: Rust-generated TypeScript representing complete fixture sites without raw Markdown parsing.

## Phase 5 — runtime and initial theme

- Implement semantic content-node rendering.
- Implement the core directive registry.
- Build page layouts and site override registry.
- Extract MamboColour tokens and MamboFont integration.
- Adapt MamboFolio's bordered cards, grid/list collections, canvas treatment, metadata, navigation, and TOC into cleaner reusable components.
- Build a documentation-oriented MamboWiki layout from the same node contract.

Deliverable: both example sites render the same content schema with distinct layout choices.

MamboFolio's possible redesign does not block compiler work. The initial theme is a reference implementation and may evolve independently.

Runtime distribution uses separately versioned `@mambosite/runtime`, `@mambosite/react`, `@mambosite/theme-default`, and framework-adapter packages. Sites pin compatible versions rather than copying versioned component directories.

## Phase 6 — website integration and deployment

- Add `mambo.toml` to MamboFolio and MamboWiki.
- Connect the optional catch-all route and `generateStaticParams()`.
- Ensure each repository contains a self-contained `docs/` tree before compilation.
- Configure static export, base paths, canonical URLs, sitemap, RSS where desired, and 404 handling.
- Add GitHub Pages build/deploy workflows.
- Verify custom-domain and repository-subpath deployments.
- Provide safe `mbsite init`, complete `mbsite build`, and GitHub Pages `mbsite deploy` commands.

Deliverable: both sites deploy from clean CI using static artifacts.

## Phase 7 — authoring workflow

- Add watch mode.
- Publish editor-agnostic example pages and templates for common content kinds.
- Add `inspect` improvements and optional editor diagnostics.
- Add client-side search generation.
- Add migration checks for legacy MamboFolio frontmatter and paths.

Deliverable: a low-friction edit, validate, preview, commit, and deploy loop for any repository-local content workflow.

## Project Mambo sync integration — outside the core

Project Mambo maintains canonical documentation in an Obsidian vault and materializes repository-local `docs/` trees with its existing sync script. Site mounts, metadata filtering, destination cleanup, and README copying are specified separately in [[Documentation Sync]].

MamboSite only consumes the resulting repository tree. The compiler does not own vault discovery, cross-repository synchronization, destination cleanup, or mount-source rewriting from authoring paths.

## Version 0.1 target acceptance criteria

- MamboWiki mounts all seven Mambo projects from its repository-local `docs/` tree without symlinks.
- MamboFolio renders pages, blog entries, project summaries, and gallery content from Markdown.
- Every public page has a deterministic route and static HTML output.
- CommonMark/GFM, supported Obsidian-compatible links/embeds, and schema-1 directives work as specified.
- Broken internal links, missing assets, ambiguous aliases, route collisions, malformed directives, and cycles fail before Next.js runs.
- Rust emits schema-valid TypeScript with no runtime Markdown parser.
- The runtime presents an initial MamboFolio-inspired theme and a usable Wiki layout.
- Both sites deploy successfully through GitHub Actions to GitHub Pages.
- A clean repeated build is deterministic.

## Explicit non-goals for version 0.1

- General-purpose CMS or web editor.
- Live preview directly inside Obsidian.
- MDX or arbitrary user components.
- Plugin API for third-party Rust parsers.
- Runtime server, database, authentication, comments, or forms backend.
- Incremental static regeneration.
- Remote content fetching.
- Automatic external-link crawling.
- Full compatibility with every Obsidian community plugin.
- Managing an Obsidian vault or synchronizing documentation between repositories.
- Advanced image transformation service.
- Incremental compilation before full-build correctness.
- A finalized permanent visual design.

## Initial decisions

| Question | Initial decision |
|---|---|
| Parser language | Rust |
| Markdown engine | Comrak behind a MamboSite adapter |
| Frontmatter YAML | `serde-saphyr` with anchors, aliases, merges, and tags disabled |
| Directive attributes | Small MamboSite-owned tokenizer and typed registry |
| Slugs/headings | Unicode NFC, ASCII lowercase, punctuation runs become `-` |
| Internal representation | Owned structured AST and content graph |
| Generated format | Typed TypeScript data modules |
| Generated React pages | No; use one generic optional catch-all route |
| Runtime Markdown parsing | No |
| Raw HTML | Disabled by default |
| Site composition | Frontmatter mounts, not symlinks |
| Embed implementation | Resolved AST nodes, not copied/indented strings |
| Default embed appearance | Bounded semantic embed component |
| Static framework | Next.js static export initially |
| Hosting | GitHub Pages via Actions artifact deployment |
| Initial visual reference | MamboFolio, with a replaceable theme contract |
| Generated output committed | Prefer no; rebuild in CI |
| First compiler mode | Deterministic full build |

## Remaining design decisions

These should be answered with fixtures or small isolated prototypes rather than production code:

### Syntax highlighting

Decide whether Rust emits highlighted token spans, the TypeScript runtime highlights at build time, or the first release uses plain semantic code blocks. Browser-time highlighting is not preferred.

### Search format

Decide whether Rust emits normalized records or a complete search index. Keep search optional and avoid coupling page rendering to one search library.

### Asset dimensions and formats

Choose the initial image formats whose intrinsic dimensions Rust reads. Unsupported formats must still copy safely when their media type is allowed.

### Additional styling adapters

The initial runtime uses generated semantic CSS variables and a typed component registry. A future styling adapter may use another CSS or component system, but it must consume the same semantic models rather than exposing utility classes in authored Markdown.

## Compatibility policy

`schema = 1` in configuration, frontmatter interpretation, directives, generated TypeScript, and runtime support form one versioned contract. The rules below are the target release policy; only schema-1 compatibility checking exists today.

- Patch releases fix bugs without intended content changes.
- Minor releases add backward-compatible optional capabilities.
- Breaking syntax or generated-schema changes require an explicit schema upgrade and migration notes.
- The compiler should support at least the current and immediately previous generated schema during migrations when practical.
- Deprecations produce targeted warnings before removal.

The specification should be updated before implementation whenever a behaviour change affects authored Markdown or generated TypeScript.

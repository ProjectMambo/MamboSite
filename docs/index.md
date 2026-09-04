---
description: Static site generator for Project Mambo websites.
title: MamboSite
order: 0
---

# MamboSite

MamboSite is a Markdown-first static site compiler for Project Mambo. It reads a clean repository-local `docs/` tree, uses Rust for parsing and validation, emits typed TypeScript modules and theme CSS, and produces a static Next.js export for deployment to GitHub Pages.

MamboSite does not require Obsidian or prescribe where authors maintain their original notes. Project Mambo uses a separate `sync-docs` workflow to export selected documents from an Obsidian vault into each repository's `docs/` tree.

The initial compiler, React runtime, default theme, Next.js adapter, and `check`, `build`, `init`, and `deploy` commands are implemented. These documents describe both the current schema-1 behavior and the remaining version-0.1 work; planned features are labeled as such.

- [[Architecture]] — system boundaries, repositories, packages, and build stages.
- [[Content Model]] — repository content structure, routing, mounts, frontmatter, and navigation.
- [[Documentation Sync]] — Project Mambo's optional Obsidian-to-repository export workflow.
- [[Markdown and Directives]] — supported Markdown and the page-component language.
- [[Parsing and Resolution]] — Rust parsing pipeline, links, embeds, and assets.
- [[TypeScript Output]] — generated module and runtime contracts.
- [[Theme and Components]] — the replaceable visual/runtime layer based initially on MamboFolio.
- [[Build and Deployment]] — compilation, Next.js static export, and GitHub Pages.
- [[Diagnostics and Testing]] — validation, error reporting, fixtures, and quality gates.
- [[Roadmap]] — current status, remaining phases, non-goals, and unresolved decisions.

The current milestone covers the content patterns used by MamboFolio, including validated content-asset publication. Fragment transclusion, advanced collection/gallery views, search, package publication, and MamboWiki integration remain planned.

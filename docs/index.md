---
description: Static site generator for Project Mambo websites.
title: MamboSite
order: 70
---

::page{layout="project" width="normal" sidebar=true}

# MamboSite

MamboSite is a Markdown-first static site compiler for Project Mambo. It reads a clean repository-local `docs/` tree, uses Rust for parsing and validation, emits typed TypeScript modules and theme CSS, and produces a static Next.js export for deployment to GitHub Pages.

MamboSite does not require Obsidian or prescribe where authors maintain their original notes. Project Mambo uses a separate `sync-docs` workflow to export selected documents from an Obsidian vault into each repository's `docs/` tree.

The initial compiler, React runtime, default theme, Next.js adapter, and `check`, `build`, `init`, and `deploy` commands are implemented. These documents describe both the current schema-1 behavior and the remaining version-0.1 work; planned features are labeled as such.

::button{label="Source code" href="https://github.com/ProjectMambo/MamboSite" variant="secondary" external=true}

## Author content

::children{include=["[[Authoring Guide]]","[[Content Model]]","[[Markdown and Directives]]","[[Theme and Components]]"] view="list" sort="order" direction="asc" show=["title","description"]}

## Configure and operate a site

::children{include=["[[Build and Deployment]]","[[Diagnostics and Testing]]","[[Documentation Sync]]"] view="list" sort="order" direction="asc" show=["title","description"]}

## Understand and extend MamboSite

::children{include=["[[Architecture]]","[[Parsing and Resolution]]","[[TypeScript Output]]","[[Roadmap]]"] view="list" sort="order" direction="asc" show=["title","description"]}

The current milestone covers the MamboFolio and MamboWiki integrations, including validated content-asset publication. Fragment transclusion, advanced collection/gallery views, search, and package publication remain planned.

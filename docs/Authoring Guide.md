---
description: End-to-end rules and copy-ready patterns for writing MamboSite content.
title: Authoring Guide
order: 5
---

# Authoring Guide

This is the starting point for a person or agent creating MamboSite pages. It covers the decisions needed to produce valid content without knowing the Rust compiler or React runtime. [[Content Model]] and [[Markdown and Directives]] remain the normative references when a rule needs more detail.

## 1. Choose the source of truth

MamboSite always compiles the repository-local directory configured by `content_root`, normally `docs/`. How files arrive there is separate.

- In a standalone site, edit `docs/` directly.
- In Project Mambo, edit the canonical Obsidian-vault copy. Project documentation belongs in `Docs/Projects/MamboXXX/`; site-owned pages and publishable media belong in `Docs/Projects/_sites/<Site>/`, with media under its `_assets/` child.
- Do not edit synchronized repository files as if they were canonical. The next sync replaces the complete `docs/` snapshot.
- Do not edit `src/generated/mambo/` or the managed directory configured by `assets_out`. A build replaces them.

See [[Documentation Sync]] for the Project Mambo export boundary. All remaining examples show the repository-facing `docs/` form that the compiler receives.

## 2. Choose a page form

| Need | Source form | Result |
|---|---|---|
| One leaf page | `docs/about.md` | `/about/` |
| A page that owns descendant pages | `docs/blog/index.md` | `/blog/` |
| A descendant | `docs/blog/first-post.md` | `/blog/first-post/` |
| Organisation without a page | `docs/reference/topic.md` without `reference/index.md` | `/reference/topic/`, but no `/reference/` page |
| Repository overview only | `README.md` | Never a site route |
| Authoring-only information | `_info.md` | Excluded everywhere |

Never create both `name.md` and `name/index.md`; they produce the same route. Prefer the leaf form until the page needs descendants. An `index.md` creates the directory route but does not display its children automatically—place a `::children` or `::gallery` directive where that collection should appear.

The configured root `docs/index.md` represents `/`. It is also the only page allowed to declare `mounts`.

## 3. Start every page with a small contract

Use frontmatter for identity, publication, routing, and structured data. Put visible layout choices in body directives.

```md
---
title: Example Page
description: One plain-text sentence describing this page.
order: 20
---

::page{layout="article" width="normal" sidebar=true}

# Example Page

Write ordinary Markdown here.
```

`title` and `description` are strongly recommended even though they can be derived. Add `order` when the page appears in an automatic collection. Use `status: draft` to omit a route from static output and `listed: false` to keep a published route out of automatic collections. Put site-specific values beneath `data`:

```yaml
data:
  period: May 2026 - Present
  githubUrl: https://github.com/ProjectMambo/MamboSite
```

Do not invent top-level frontmatter keys. Strict sites reject them; non-strict sites retain them only as compatibility data. The complete field table and ordering rules are in [[Content Model]].

The default renderer generates one H1 from `title` when the body has neither an H1 nor a title-rendering `::hero`. A body H1 suppresses that generated title. A `::hero` with its default `show-title=true` also suppresses it, so do not add a duplicate H1 after such a hero.

## 4. Use a copy-ready page pattern

### Article or documentation page

```md
---
title: Installation
description: Install and verify the project.
order: 10
---

::page{layout="docs" width="normal" sidebar=true}

# Installation

## Requirements

Content.

## Install

Content.

## Verify

Content.
```

With `sidebar=true`, H2 through H4 headings produce an automatic table of contents. A page without matching headings gets no empty TOC. On small screens the automatic TOC is a closed disclosure before the article; on larger screens it becomes a sticky, independently scrolling rail that follows the current section. Use an authored `::toc{...}` only when the TOC must appear at a specific body position; it suppresses the automatic copy.

### Collection index and children

```text
docs/projects/
├── index.md
├── MamboSite.md
└── MamboWiki.md
```

```md
---
title: Projects
description: Projects in this collection.
---

::page{layout="collection" width="wide" sidebar=false}

# Projects

::children{view="grid" columns=3 depth=1 sort="order" direction="asc" show=["cover","title","description"]}
```

Each child controls its position with `order`, whether it is included with `listed`, and its preview image with `cover`.

### Project page

```md
---
title: MamboSite
description: Markdown-first static site compiler.
tags: [Project Mambo, Rust]
data:
  period: May 2026 - Present
  githubUrl: https://github.com/ProjectMambo/MamboSite
---

::page{layout="project" width="normal"}

::meta{show=["period","description","tags","githubUrl"] style="stack" empty="hide"}

## Overview

Content.
```

### Gallery index and item

Use an index page whose direct children each provide a `cover`:

```md
---
title: Gallery
description: Selected work.
---

::page{layout="gallery" width="wide" sidebar=false}

# Gallery

::gallery{source="children" view="grid" columns=3 fit="cover" captions=true}
```

A gallery item can use the same asset for its preview and hero:

```md
---
title: Mambo
description: Gallery item description.
cover: assets/mambo.png
---

::page{layout="gallery" width="wide"}

::hero{image="assets/mambo.png" align="center" show-description=true}
```

### Home page and action grid

The default theme reads three optional `data` objects. `navigation` belongs on the entry page; its first item becomes the brand control and the remaining items become navigation links. `footer` also belongs on the entry page. `hero` may belong on any page with a `::hero`.

```yaml
data:
  navigation:
    - label: MY SITE
      href: /
    - label: HOME
      href: /
    - label: PROJECTS
      href: /projects/
  hero:
    quote: A short quotation.
    attribution: Its source
  footer:
    copyright: 2026 My Site
    links:
      - label: Source Code
        href: https://github.com/example/site
```

The footer adds the copyright symbol. Contact and action collections use ordinary links inside responsive columns:

```md
::::columns{count=3 gap="small" collapse-at="md"}

:::column

::button{label="Email" href="mailto:hello@example.com" variant="card" external=true}

:::

:::column

::button{label="GitHub" href="https://github.com/example" variant="card" external=true}

:::

:::column

::button{label="Projects" href="/projects/" variant="card"}

:::

::::
```

## 5. Link pages and assets

Use either normal Markdown links or Obsidian-style note links:

```md
[Install](Install.md)
[[Install]]
[[Install#Verify|verification steps]]
![[Shared Introduction]]
```

Relative `.md` links and wikilinks resolve through the compiled content graph, where unsafe URL schemes fail validation. Prefer route links such as `/projects/` for directive properties that expect a page route. For button actions, use ordinary site routes or `https:`, `mailto:`, and `tel:` destinations; the runtime refuses unsafe schemes, but directive targets are not compiler-resolved yet.

Content media has one root-relative author namespace. Store a direct-repository asset at `docs/_assets/profile/photo.jpg` and refer to it from any page as `assets/profile/photo.jpg`:

```md
![Portrait](assets/profile/photo.jpg)
```

The same namespace works in `cover`, `hero.image`, `button.href`, wikilinks, and Obsidian embeds. Do not write the generated `/mambo/assets/...` URL or deployment base path in source Markdown. A site may also keep icons and social previews in `_assets/` and point its renderer metadata at their generated URLs; fonts may remain theme-package or site-owned files.

## 6. Mount project documentation

Mounts compose repository-local trees into the public route hierarchy without symlinks. Only the configured entry may declare them:

```yaml
---
title: Wiki
mounts:
  - path: /mambosite
    source: _mounts/mambosite/index.md
---
```

The source must be an `index.md` inside the content root. Its directory becomes the mount root, so `_mounts/mambosite/Commands.md` becomes `/mambosite/commands/`. Project Mambo's canonical vault entry uses a vault wikilink as the source; `sync_docs.js` materializes the directory and rewrites only the exported entry. Do not put a vault path into a repository-local entry manually.

## 7. Stay inside current renderer support

The compiler recognizes some forward-looking values so sites can override components, but the default theme does not implement every accepted mode. Unless the site supplies an override, use this current subset:

| Feature | Use now | Do not generate for the default theme yet |
|---|---|---|
| `children` | `depth=1`; `list`, `grid`, `cards`, `hidden` | recursive depth, `tree`, `table` |
| `gallery` | children of the current or referenced index; `grid` | `page-embeds`, arbitrary non-page folders, `masonry`, `carousel` |
| note embed | whole page | heading or block fragment transclusion |
| `include` | whole-page source; `embed` or `inline` | fragment source |
| buttons | safe web URLs and ordinary site routes | compiler-authoritative note-style directive targets |
| raw HTML | displayed as code text | trusted HTML rendering |

Use [[Markdown and Directives]] for the exact directive grammar, property types, defaults, and contexts. Never copy a planned value from [[Roadmap]] into authored content as though it already renders.

## 8. Validate the result

For a direct repository workflow:

```bash
mbsite check
npm run dev
mbsite build
```

For Project Mambo, synchronize the canonical change before validating its destination. Follow [[Documentation Sync#After every sync]] to include every affected repository or site mount, review the materialized files, and either commit and publish them or hold them for manual review. MamboSite itself uses `mbsite check`; local preview, full site builds, and deployment apply only to website repositories with a configured renderer.

Before finishing an authored change, verify:

- The correct canonical source was edited.
- No leaf/folder route pair collides.
- New pages are reachable from an index, collection, or link.
- All local links, embeds, mount sources, and `assets/...` references exist.
- Only current default-renderer modes are used unless an override is known.
- `mbsite check` exits successfully.
- Generated TypeScript and managed public assets were not hand-edited.

These checks are also the minimum contract for an automated authoring agent.

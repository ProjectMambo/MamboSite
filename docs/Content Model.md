---
description: Rules for repository content roots, pages, routes, mounts, and metadata.
title: Content Model
order: 20
---

# Content Model

This document is the normative schema-1 content contract. For a page-form decision table and complete copy-ready examples, start with [[Authoring Guide]].

## Repository content root

MamboSite compiles a self-contained directory inside the website repository. The normal configuration is explicit:

```toml
content_root = "docs"
entry = "index.md"
```

`content_root` is resolved relative to `mambo.toml`. `entry` is resolved relative to that content root and is never inferred from the author's editor or directory history. In the first release the configured entry must be an `index.md`; its route is `/`.

A wiki-style repository can use this compiler-facing structure:

```text
MamboWiki/
├── mambo.toml
├── README.md                 # repository README; not site content by default
└── docs/                     # content_root
    ├── index.md              # configured site entry
    ├── about.md              # site-owned page
    ├── _assets/             # non-routable content assets
    │   └── logo.png
    └── _mounts/              # source storage, not an automatic route segment
        ├── mambodot/
        │   ├── index.md
        │   ├── Commands.md
        │   └── Keybinds.md
        └── mambosite/
            ├── index.md
            └── Architecture.md
```

The compiler does not require any particular authoring tool and does not create this tree from another source. Authors may edit `docs/` directly or populate it with an external workflow. Project Mambo's Obsidian-vault synchronization is one such workflow and is specified separately in [[Documentation Sync]].

Site-owned pages and mounted project documentation may therefore have different origins before synchronization, but they are ordinary repository-local files by the time MamboSite starts.

## Page forms

MamboSite supports both leaf files and folder pages:

```text
blog/article.md              -> /blog/article/
blog/article/index.md        -> /blog/article/
blog/index.md                -> /blog/
```

Rules:

- Use `name.md` for a leaf page.
- Use `name/index.md` when the page has child pages or related authoring material.
- An `index.md` represents its containing directory.
- `name.md` and `name/index.md` may not coexist because they produce the same route.
- A directory without `index.md` is organisational only. Its descendants still receive routes, but the directory itself has no page.
- Only the configured site-entry `index.md` may declare mounts in the first release. Any `index.md` may use child-layout directives.

Routing children does not render them. An index page must place `::children` or `::gallery` in its body when and where those descendants should be visible; omitting a collection directive leaves the child routes reachable only through other links.

## Discovery and exclusions

The compiler discovers UTF-8 files with the exact `.md` extension beneath the configured content root. It performs two distinct forms of discovery:

1. **Ordinary discovery** finds routable site-owned pages while skipping reserved trees such as `_mounts/`.
2. **Mount discovery** begins at each valid `source` declared by the configured site entry and admits that source index plus its descendants.

The second pass is an explicit reachability rule, not a general exception that makes every file beneath `_mounts/` routable.

The following are excluded by default:

- Any path segment beginning with `_` during ordinary page discovery. An explicit mount may enter `_mounts/`; the asset pass reads only the reserved `_assets/` root.
- `_info.md` anywhere, including inside a mounted subtree.
- `README.md` anywhere. A repository or project README may coexist with documentation but is not a web page unless a future configuration explicitly opts it in.
- Hidden filesystem entries beginning with `.`.
- `archive/`, unless explicitly included later by configuration.
- Obsidian configuration, templates, canvas files, bases, and plugin data.

Draft pages are still compiled into schema-1 output so links and graph data remain deterministic. The Next adapter omits them from `generateStaticParams()`, and the default runtime filters them from child, related, and backlink collections. A future compiler mode may omit drafts entirely.

The configured entry is admitted explicitly even though ordinary discovery rules are evaluated separately. `_info.md` is always authoring-only and cannot be made routable by linking to it or placing it below a mount.

Code must not rely on directory iteration order. All discovered paths are normalized to forward-slash, content-root-relative logical paths and sorted before processing.

## Route derivation

Route segments come from directory and file names unless `slug` overrides the final page segment.

Default segment normalization:

1. Normalize Unicode to NFC.
2. Trim surrounding whitespace.
3. Convert ASCII letters to lowercase.
4. Treat punctuation, spaces, and underscores as separators.
5. Collapse repeated `-` characters.
6. Remove leading and trailing `-` characters.
7. Preserve non-ASCII letters and encode them normally in URLs.

Examples:

```text
My Project Notes.md   -> /my-project-notes/
blog/Release Notes.md -> /blog/release-notes/
第二卷/index.md         -> /第二卷/
```

The source path is not the final route for mounted content. Mount resolution first assigns the route prefix, then normal relative route derivation continues below the mounted `index.md`.

Every route must:

- Begin and end with `/` when `trailing_slash = true`.
- Be unique after normalization.
- Avoid `.` and `..` segments.
- Avoid URL query and fragment characters in path segments.
- Remain stable unless the author changes the path or explicit `slug`.

Route collisions are build errors and must list every contributing source file.

## Site entries and mounts

A site entry is an `index.md` selected by `mambo.toml`. Its own route is `/`.

Mounts live in that entry's frontmatter because they define the source graph rather than a visible component. `source` is a forward-slash, content-root-relative path to a repository-local `index.md`:

```yaml
---
title: Project Mambo Wiki
mounts:
  - path: /mambodot
    source: _mounts/mambodot/index.md
  - path: /mambosite
    source: _mounts/mambosite/index.md
  - path: /mambowiki
    source: _mounts/mambowiki/index.md
---
```

Mount rules:

- `source` is resolved relative to the configured content root, not relative to the entry file.
- `source` must remain within the content root after lexical normalization and filesystem canonicalization. Absolute paths, escaping `..` traversal, and symlinks that escape the root are errors.
- `source` must resolve to an `index.md`.
- The source index's directory becomes the mounted content root.
- `path` must be an absolute site route other than `/`.
- All descendant pages are mounted below `path` while preserving their relative hierarchy.
- `_mounts` and other storage segments before the source index never become URL segments.
- A source may be mounted once per site in the first release.
- Mount paths must not overlap physical routes or other mount paths ambiguously.
- Only the configured site entry may declare mounts in the first release. A `mounts` field on any other page is an error.
- A site's own project documentation may be mounted normally from a distinct repository-local subtree such as `_mounts/mambowiki/index.md`; this is not a self-reference to `docs/index.md`.
- Mount declarations do not control visual ordering. Use `order`, `children`, or explicit navigation data for that.

Mount resolution operates on repository-local logical source paths, never on authoring-workflow paths. No symlink is required or implied.

## Frontmatter

Frontmatter is YAML delimited by `---` lines at the start of the file. It contains identity, publication, routing, and source-graph data. Visual composition belongs in body directives.

Core fields:

| Field | Type | Default | Meaning |
|---|---|---|---|
| `title` | string | first H1 or filename | Display and document title |
| `description` | string | first suitable paragraph | Plain-text summary |
| `slug` | string | derived segment | Final route segment override |
| `status` | `published` or `draft` | `published` | Publication intent used by adapters and collections |
| `listed` | boolean | `true` | Inclusion in automatic child/related collections |
| `date` | string | none | Original publication date |
| `updated` | string | none | Last meaningful content update |
| `tags` | string or string array | empty | Related-content input |
| `aliases` | string or string array | empty | Additional internal-link targets |
| `order` | integer | none | Explicit sibling ordering |
| `cover` | link/path | none | Default preview or hero asset |
| `mounts` | mount array | empty | Site-entry source mappings |
| `data` | mapping | empty | Project-specific metadata passed to TypeScript |

Empty strings normalize to absent values. Authors should use ISO 8601 dates because the default runtime sorts and formats those strings, but schema 1 does not yet validate date syntax. Tags are retained as authored strings; normalized taxonomy IDs are planned rather than generated today.

Unknown top-level keys are errors when `[frontmatter].strict = true` because they are commonly misspellings. Schema 1 defaults this option to `false` for migration compatibility; new sites and automated authoring workflows should enable it once their content uses only the documented fields. Site-specific values such as `period`, `githubUrl`, or `wikiUrl` belong beneath `data`:

```yaml
data:
  period: May 2026 - Present
  githubUrl: https://github.com/ProjectMambo/MamboSite
```

Authoring-workflow fields may be declared as ignored compatibility fields in configuration during migration. The defaults ignore `created`, `categories`, and `project`; `legacy_data_fields` currently promotes `period`, `wikiUrl`, and `githubUrl` into `data`. These fields do not silently acquire other MamboSite semantics. Project Mambo's synchronization-time frontmatter handling is documented in [[Documentation Sync]] and is not part of the compiler language.

## Title and description derivation

Title precedence:

1. Frontmatter `title`.
2. First level-one heading.
3. Parent folder name for `index.md`, otherwise file stem.

Description precedence:

1. Frontmatter `description`.
2. First plain paragraph that is not a directive, callout, or embed.
3. Absent.

The renderer decides whether the body H1 is shown. The compiler keeps it in the body AST and reports a warning when multiple level-one headings appear.

## Children and ordering

A page's direct children are routes whose nearest existing ancestor page is that page. Physical children and mounted children participate in the same route hierarchy.

Default ordering:

1. Pages with numeric `order`, ascending.
2. Pages without `order`, sorted by locale-independent normalized title.
3. Logical source path as a deterministic tie-breaker.

The compiler records all direct child IDs, including draft and unlisted pages. The current Next adapter excludes drafts from static parameters, while default child and related components filter draft pages and hide unlisted pages unless `include-unlisted` permits them. Header navigation is explicit `data.navigation`; generated navigation and search indexes do not exist yet.

## Content graph

The compiler constructs a graph rather than treating each Markdown file independently. Current records cover pages, headings, blocks, routes, mounted sources, parent-child relationships, links, embeds, and backlinks.

This graph supplies:

- Route lookup.
- Children and runtime collection queries.
- Backlinks.
- Related content.
- Embed cycle and depth validation.
- Related-content inputs.

Richer asset metadata, a nested mount graph, generated navigation/search model, and an incremental dependency graph are planned.

## Self-contained content root contract

MamboSite begins with an already populated content root and has no dependency on a vault, note database, export script, or editor. For a build to be reproducible, `docs/` must contain every local input needed by the active site:

- The configured `index.md` site entry.
- Every ordinarily discovered site-owned page.
- Every mounted source index and its descendant pages.
- Every local note reached by a link or embed that must resolve in the site.
- Every published content asset under `_assets/`; site icons and fonts may remain site-owned files outside the managed `assets_out` subtree of `public/`.

All logical note and mount paths are interpreted within this root. A build does not search parent directories, a home directory, a known vault location, sibling repositories, or the network to repair missing content.

Schema 1 gives content assets one explicit namespace. Authors reference `assets/<path>` from any page, while the repository stores the matching file at `_assets/<path>`. The compiler validates containment and existence, rewrites the reference to the public URL below `assets_out/assets/`, and publishes every regular file in `_assets/` with the generated theme. Asset paths are independent of the page's directory.

Other local or absolute paths are not claimed by this namespace. This lets a site keep icons and fonts elsewhere in its own `public/` tree; files inside `assets_out` are managed and replaced on build. Content assets are copied without hashing, transformation, media inspection, or deduplication in schema 1.

An example complete input is:

```text
docs/
├── index.md
├── about.md
├── _assets/
│   └── logo.png
└── _mounts/
    ├── mambodot/
    │   ├── index.md
    │   └── Commands.md
    └── mambosite/
        ├── index.md
        └── Architecture.md
```

Files outside `docs/`, including the repository-root `README.md`, do not participate in parsing or resolution by default. How another system materializes or refreshes this tree is intentionally outside the compiler contract; see [[Documentation Sync]] for Project Mambo's workflow.

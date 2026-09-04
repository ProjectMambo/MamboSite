---
description: Project Mambo's optional Obsidian-to-repository documentation workflow.
title: Documentation Sync
order: 25
---

# Documentation Sync

## Scope

MamboSite compiles a self-contained `docs/` directory inside a repository. It does not require Obsidian, know where the original notes live, or synchronize content itself.

Project Mambo uses an additional authoring adapter because its project documentation is maintained centrally in an Obsidian vault. The existing Templater user script at `Scripts/sync_docs.js` materializes the repository-facing structure. This document specifies that Project Mambo workflow; it is not a requirement for other MamboSite users.

```text
Obsidian vault                         Repository                         MamboSite
------------------------------         -----------------------------      -----------------
Docs/Projects/<Project>/        ---->  README.md + docs/           ---->  compile docs/
Docs/Projects/_sites/<Site>/    ---->  site docs + local mounts           ignore vault layout
Docs/assets/<Site>/             ---->  docs/_assets/                       publish content assets
```

Other users may edit `docs/` directly, generate it from another system, or implement a different synchronizer. The only compiler requirement is the final repository-local content contract described in [[Content Model]].

## Vault ownership model

The vault separates project ownership from site composition:

```text
Docs/
├── assets/
│   └── MamboFolio/              # publishable portfolio media
└── Projects/
    ├── MamboColour/             # canonical documentation for one project
    │   ├── index.md             # project landing page when mounted
    │   ├── README.md            # repository README source
    │   └── Commands.md
    ├── MamboDot/
    ├── MamboFinance/
    ├── MamboFolio/
    ├── MamboFont/
    ├── MamboSite/               # MamboSite's own documentation
    ├── MamboWiki/
    └── _sites/
        ├── MamboFolio/          # portfolio-owned pages and route hierarchy
        │   ├── index.md
        │   ├── About.md
        │   ├── blog/
        │   ├── gallery/
        │   └── project/
        └── MamboWiki/           # wiki root and mount declarations
            └── index.md
```

Each `Docs/Projects/MamboXXX/` directory is the single authored copy of that project's documentation. A `_sites/<Site>/` directory contains only pages owned by the site itself. It may publish canonical project documentation through mounts without nesting the authored project directories or creating symlinks.

This also resolves the MamboWiki self-documentation case. `Docs/Projects/_sites/MamboWiki/index.md` is the website entry, while `Docs/Projects/MamboWiki/index.md` is the canonical MamboWiki project page. They are different source files, so the latter can be mounted at `/mambowiki/` without a filesystem loop.

## Repository profiles

`Scripts/sync_docs.js` has one profile for each Mambo repository. A profile always declares:

- A canonical project source at `Docs/Projects/<Project>/`.
- A repository destination at `~/ProjectMambo/<Project>/`.
- Optionally, a site source at `Docs/Projects/_sites/<Site>/`.
- Optionally, an explicit asset source below `Docs/assets/`.

Current site profiles are MamboFolio and MamboWiki. MamboSite is an ordinary project profile: its documentation fills `MamboSite/docs/`, while its Cargo/npm workspaces, tests, templates, and other source files remain untouched. If a repository later needs both site-owned pages and mounts, adding `siteSource` changes only how that repository's `docs/` is assembled.

The Obsidian command palette exposes both the full `Sync Docs` command and a targeted `Sync MamboFolio Docs` command. The same adapter may be run from a terminal for validation or automation:

```bash
node Scripts/sync_docs.js --sync MamboFolio
node Scripts/sync_docs.js --sync-all
```

A targeted sync stages and replaces only the named repositories and skips standalone exports. Unknown or empty selections fail before a destination is changed.

## Ordinary repository export

For an ordinary project, the complete canonical directory becomes the repository's `docs/` directory:

```text
vault                                  MamboSite repository
--------------------------------       --------------------------------
Docs/Projects/MamboSite/        ---->  docs/
  index.md                              index.md
  README.md                             README.md
  Architecture.md                      Architecture.md
  ...                                  ...

Docs/Projects/MamboSite/README.md ---> README.md
```

The root `README.md` is a second transformed copy of the canonical project README. Canonical inline links use root-relative `docs/...` destinations; the copy inside `docs/` removes that prefix so both copies resolve locally. The repository root itself is never cleaned; source code and configuration outside `docs/` are not touched.

Within a MamboSite content root, `README.md` is repository documentation and is non-routable by default. `index.md` remains the publishable landing page.

## Site repository export

A site profile uses its `_sites/<Site>/` directory for physical pages, then adds materialized mount sources:

```text
MamboWiki/
├── README.md                     # from Docs/Projects/MamboWiki/README.md
├── docs/                         # replaced as one generated snapshot
│   ├── index.md                  # from Docs/Projects/_sites/MamboWiki/index.md
│   └── _mounts/                  # generated; never authored here
│       ├── mambocolour/
│       ├── mambodot/
│       ├── mambofinance/
│       ├── mambofolio/
│       ├── mambofont/
│       ├── mambosite/
│       └── mambowiki/
├── src/                          # untouched website implementation
└── package.json                  # untouched
```

MamboFolio follows the same profile shape but currently declares no mounts. Its portfolio pages come only from `Docs/Projects/_sites/MamboFolio/`, while the canonical MamboFolio README still supplies the repository root README.

`docs/_mounts/` is a reserved generated namespace. MamboSite excludes it from ordinary route discovery and enters its contents only through explicit `mounts` declarations, so storage paths such as `/_mounts/mambodot/` never become public routes.

## Mount materialization and rewriting

The vault-facing site entry may use an Obsidian wikilink to point to a canonical project entry:

```yaml
---
title: Project Mambo Wiki
mounts:
  - path: /mambodot
    source: "[[Docs/Projects/MamboDot/index]]"
  - path: /mambosite
    source: "[[Docs/Projects/MamboSite/index]]"
---
```

For each mount, the sync script:

1. Resolves `source` to a canonical `index.md` beneath `Docs/Projects/` and rejects `_sites/`, fragments, aliases, missing files, and paths outside that root.
2. Copies the source index's complete containing directory into `docs/_mounts/<mount-path>/`.
3. Rewrites only the staged/exported site entry so its source is repository-local.
4. Leaves the vault entry unchanged.

The exported result is:

```yaml
---
title: Project Mambo Wiki
mounts:
  - path: /mambodot
    source: "_mounts/mambodot/index.md"
  - path: /mambosite
    source: "_mounts/mambosite/index.md"
---
```

The current adapter accepts the conventional YAML block-list form shown above. This is an implementation constraint of `sync_docs.js`, not a limitation on MamboSite's complete YAML frontmatter parser.

Mount storage is derived from the route path. Nested paths remain readable—for example, `/projects/mambodot` materializes at `_mounts/projects/mambodot/`. Duplicate, case-colliding, root, or overlapping mount paths fail before any destination is cleaned.

Mounted content is copied once and is not recursively interpreted by the sync script. A mounted project's own `index.md` therefore cannot trigger a copy loop. MamboSite later resolves its links, embeds, routes, and child hierarchy during compilation.

## Markdown transformation

Ordinary exported Markdown files have only these top-level frontmatter properties removed:

```yaml
created: ...
updated: ...
project: ...
```

The ordinary-page transformation is intentionally narrow:

- A selected field and its indented continuation lines are removed.
- Nested fields with the same name are preserved.
- All other fields are preserved, including `title`, `description`, `categories`, `tags`, `mounts`, `data`, and custom page metadata.
- If no frontmatter fields remain, the empty delimiters are removed.
- When removing the complete block, leading blank separator lines are removed with it; authored Markdown is not globally trimmed by MamboSite.
- Markdown bodies and non-Markdown files are otherwise copied as authored.

`README.md` is the exception. Every file whose exact basename is `README.md` has its complete leading YAML frontmatter block removed, whether it is copied into `docs/`, to a repository root, or through a standalone file export. Inline `docs/...` links are localized only in the `docs/README.md` copy. A byte-order mark is also removed. If a README begins a frontmatter block without closing it, the sync fails before replacing any destination.

Any file whose exact basename is `_info.md` is excluded at every depth. It is vault organisation metadata, not repository documentation or site content.

## Clean replacement and safety

Stale documentation must not survive a sync, so each configured `<repository>/docs/` is replaced as a complete snapshot. The script follows this order:

1. Resolve and validate every configured source, repository, README, site entry, and mount.
2. Build every repository's new documentation in a temporary staging directory inside that repository.
3. Stop without cleaning current targets if validation or staging fails.
4. Verify that the target is exactly the configured repository's direct `docs/` child and is not a symlink.
5. Remove that `docs/` once and rename the completed staged tree into place.
6. Replace the repository root `README.md` as a single file; never clean the repository root.
7. Run standalone exact-file exports, such as the Project Mambo GitHub profile README, without cleaning their parent directories.

Source and destination symlinks are rejected rather than followed. Missing configured inputs are errors instead of silently producing a partial successful sync.

The operation is deterministic for identical vault inputs. It is safe to run repeatedly; removed source files disappear from the next repository snapshot because the destination `docs/` is rebuilt rather than merged.

## Assets and external dependencies

A site profile may name one explicit source below `Docs/assets/`. MamboFolio currently maps the contents of `Docs/assets/MamboFolio/` byte-for-byte to the reserved repository directory `docs/_assets/`. The staged site source may not also contain `_assets/`, so two inputs can never silently overwrite each other.

```text
Vault                                  Repository             Published by mbsite build
Docs/assets/MamboFolio/profile/a.jpg   docs/_assets/profile/a.jpg   public/mambo/assets/profile/a.jpg
```

Markdown and directives refer to that file as `assets/profile/a.jpg`. This is a MamboSite root-relative content namespace, not a path relative to the current note directory. The compiler validates and rewrites it after sync. Site icons and fonts remain site-owned files outside the managed `assets_out` subtree of `public/` and are not part of this asset source.

The synchronizer never crawls the private vault for attachments. A profile without `assetSource` exports no external asset tree, and a configured source must remain below `Docs/assets/` and contain no symlinks. The complete `docs/` replacement removes stale synchronized assets.

## Separation of responsibilities

The boundary is deliberate:

| Concern | `sync_docs.js` | MamboSite |
|---|---:|---:|
| Choose Project Mambo vault sources | Yes | No |
| Strip vault-only metadata | Yes | No |
| Exclude `_info.md` | Yes | Also ignores it defensively |
| Materialize external project folders | Yes | No |
| Rewrite vault mount sources to repository-local paths | Yes | No |
| Parse Markdown and directives | No | Yes |
| Resolve repository-local mounts, note links, and note embeds | No | Yes |
| Select and materialize an explicit Vault asset source | Yes | No |
| Validate asset references and publish content assets | No | Yes |
| Derive routes and child relationships | No | Yes |
| Generate TypeScript and static site data | No | Yes |

This keeps MamboSite portable: its tests and CI operate only on repository fixtures, while Project Mambo retains one convenient source of truth in Obsidian.

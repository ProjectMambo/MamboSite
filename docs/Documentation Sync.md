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
Docs/Projects/_sites/<Site>/    ---->  site docs, _assets, mounts          ignore vault layout
```

Other users may edit `docs/` directly, generate it from another system, or implement a different synchronizer. The only compiler requirement is the final repository-local content contract described in [[Content Model]].

Use [[Authoring Guide]] for page forms and copy-ready content patterns. This document covers only how Project Mambo turns its canonical copies into that compiler-facing structure.

## Vault ownership model

The vault separates project ownership from site composition:

```text
Docs/
└── Projects/
    ├── MamboColour/             # canonical documentation for one project
    │   ├── index.md             # project landing page when mounted
    │   ├── README.md            # repository README source
    │   └── Commands.md
    ├── MamboDocs/
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
        │   ├── _assets/         # publishable portfolio media
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

Current site profiles are MamboFolio and MamboWiki. MamboSite is an ordinary project profile: its documentation fills `MamboSite/docs/`, while its Cargo/npm workspaces, tests, templates, and other source files remain untouched. If a repository later needs both site-owned pages and mounts, adding `siteSource` changes only how that repository's `docs/` is assembled.

The Obsidian command palette exposes both the full `Sync Docs` command and a targeted `Sync MamboFolio Docs` command. The same adapter may be run from a terminal for validation or automation:

```bash
node Scripts/sync_docs.js --sync MamboFolio
node Scripts/sync_docs.js --sync-all
```

A targeted sync stages and replaces only the named repositories and skips standalone exports. Unknown or empty selections fail before a destination is changed.

## After every sync

The synchronizer only materializes files. It does not validate, stage, commit, push, or deploy them. Start with a clean working tree in every selected destination: its synchronized `README.md` and complete `docs/` snapshot are replaced, so uncommitted edits there can be lost.

A targeted sync updates only the named repositories. `--sync-all` updates every repository profile and the standalone exports. A site that mounts changed project documentation must also be synchronized; for example, refreshing MamboSite and its MamboWiki mount together uses:

```bash
node Scripts/sync_docs.js --sync MamboSite MamboWiki
```

For each changed destination, run its documented validation commands. A repository with `mambo.toml` should pass `mbsite check`. Then review and commit only the synchronized paths:

```bash
cd ~/ProjectMambo/<Project>
git status --short
git diff -- README.md docs/
git diff --check
git add -- README.md docs/
git diff --cached --check
git diff --cached -- README.md docs/
git commit -m "docs: sync documentation"
```

If the staged diff is empty, skip the commit. If the change requires manual review, stop after local validation and the optional commit; do not push or deploy it.

For an ordinary repository, including MamboSite itself, push the reviewed commit explicitly with `git push origin main`. For a configured website repository, run `npm run deploy` instead of a separate push. `mbsite deploy` requires a clean deployment branch, builds locally, and either pushes the new commit or dispatches the Pages workflow when that commit is already remote. Pushing first and then running it may start a duplicate deployment.

If synchronization fails, inspect every selected destination before retrying or committing. All replacement trees are staged first, but final repository replacements happen sequentially; a late filesystem failure can leave an earlier destination updated without rolling it back.

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

The root `README.md` is a second transformed copy of the canonical project README. In the copy inside `docs/`, `docs/...` destinations lose that prefix while other repository-relative destinations gain `../`; absolute URLs, fragments, queries, root-relative paths, and existing parent-relative paths stay unchanged. The repository root copy is not link-rewritten. The repository root itself is never cleaned; source code and configuration outside `docs/` are not touched.

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
│       ├── mambodocs/
│       ├── mambodot/
│       ├── mambofinance/
│       ├── mambofolio/
│       ├── mambofont/
│       ├── mambosite/
│       └── mambowiki/
├── src/                          # untouched website implementation
└── package.json                  # untouched
```

MamboFolio follows the same profile shape but currently declares no mounts. Its portfolio pages and `_assets/` tree come only from `Docs/Projects/_sites/MamboFolio/`, while the canonical MamboFolio README still supplies the repository root README.

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
2. Copies the source index's containing directory into `docs/_mounts/<mount-path>/`, excluding non-routable `README.md` files and vault-only `_info.md` files at every depth.
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

`README.md` is the exception. Each repository README has its complete leading YAML frontmatter block removed when copied into `docs/`, to a repository root, or through a standalone file export. The `docs/README.md` copy localizes both Markdown destinations and HTML `href`/`src` attributes: it removes a leading `docs/` or `./docs/`, prefixes other repository-relative paths with `../`, and preserves destinations that are already absolute or parent-relative. Mounted trees omit `README.md` files because they are non-routable and their repository-relative links have no stable meaning inside `_mounts/`. A byte-order mark is also removed. If a README begins a frontmatter block without closing it, the sync fails before replacing any destination.

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

A site's publishable media lives beside its pages in the site's canonical `_assets/` directory. The site source is copied as one tree, so MamboFolio maps `Docs/Projects/_sites/MamboFolio/_assets/` byte-for-byte to the reserved repository directory `docs/_assets/`.

```text
Vault                                                     Repository                    Generated by mbsite build        Static artifact
Docs/Projects/_sites/MamboFolio/_assets/profile/a.jpg  -> docs/_assets/profile/a.jpg -> public/mambo/assets/profile/a.jpg -> out/mambo/assets/profile/a.jpg
```

Markdown and directives refer to that file as `assets/profile/a.jpg`. This is a MamboSite root-relative content namespace, not a path relative to the current note directory. After sync, the compiler validates the matching `docs/_assets/` file, rewrites its URL to the configured `assets_out/assets/` path, and includes the generated file in the static export. For the default `assets_out = "public/mambo"`, that means `/mambo/assets/...` before the renderer adds `site.base_path`.

The synchronizer never searches elsewhere in the private vault for attachments. A site without `_assets/` exports no content assets. Source symlinks are rejected, and the complete `docs/` replacement removes stale synchronized assets. Project Mambo sites keep icons and social previews in this same canonical `_assets/` tree, then point their renderer metadata at the generated `assets_out/assets/` URLs. MamboFont remains bundled by the MamboSite default-theme package.

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
| Copy the site's `_assets/` tree into repository docs | Yes | No |
| Validate asset references and publish content assets | No | Yes |
| Derive routes and child relationships | No | Yes |
| Generate TypeScript and static site data | No | Yes |

This keeps MamboSite portable: its tests and CI operate only on repository fixtures, while Project Mambo retains one convenient source of truth in Obsidian.

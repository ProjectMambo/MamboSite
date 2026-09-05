---
description: Repository-local content, Rust compilation, Next.js static generation, and GitHub Pages deployment.
title: Build and Deployment
order: 60
---

# Build and Deployment

## End-to-end flow

MamboSite starts from a self-contained content tree inside the website repository. How that tree is authored or synchronized is deliberately separate from compilation:

```text
1. Prepare the repository-local docs/ tree
2. Parse and validate docs/ with Rust
3. Generate TypeScript, theme CSS, and content assets
4. Render through the versioned MamboSite React runtime and build a static export
5. Upload the static artifact to GitHub Pages
```

The website repository must contain everything CI needs. MamboSite does not require access to an Obsidian vault, another notes repository, or any authoring-time source.

## Repository-local content input

The compiler reads one configured content root, normally `docs/`:

```text
MamboWiki/
├── README.md
├── docs/
│   ├── index.md
│   ├── about.md
│   ├── _assets/
│   └── _mounts/
│       ├── mambodot/
│       └── mambosite/
├── mambo.toml
└── src/
```

Before `mbsite check` or `mbsite build` runs, `docs/` must contain:

- The configured entry page.
- Every physical site page.
- Every mounted documentation tree.
- Every local link or embed target required by the site.
- Every `assets/...` target under the matching `_assets/` path.

Content assets use a fixed mapping: authored `assets/<path>` resolves to repository `_assets/<path>` and is published under the managed `assets_out/assets/` directory. Site-owned icons and fonts can remain ordinary files elsewhere in `public/`; manual files must stay outside `assets_out`, which MamboSite replaces as a managed tree.

All compiler paths are interpreted relative to this content root. Mounted copies may live under an excluded implementation directory such as `_mounts/`; an explicit mount makes their pages public at the mount's configured route. The compiler must not depend on where these files lived before they entered the repository.

Project Mambo happens to author canonical project documentation in an Obsidian vault and uses a separate sync script to materialize each website's `docs/` tree. It is an integration around MamboSite, not a MamboSite command or part of the compiler contract. See [[Documentation Sync]] for that workflow, including README copying, metadata filtering, clean replacement, and mount rewriting.

Other users may edit `docs/` directly, generate it with another tool, use Git submodules before compilation, or maintain it in any other way. Whatever the method, CI sees the same repository-local input contract.

## Website configuration

Each website repository has one `mambo.toml`:

```toml
schema = 1
content_root = "docs"
entry = "index.md"
typescript_out = "src/generated/mambo"
assets_out = "public/mambo"

[site]
title = "Project Mambo Wiki"
url = "https://projectmambo.org"
base_path = ""
trailing_slash = true
language = "en-SG"

[renderer]
enabled = true
kind = "next"
package_manager = "npm"
build_script = "mambosite:render"
output_dir = "out"

[deploy]
remote = "origin"
branch = "main"
workflow = ".github/workflows/pages.yml"
```

`--config` chooses the TOML file; omitted fields use schema defaults. The compiler writes `site.base_path`, `site.url`, `site.trailing_slash`, and other site metadata into the generated manifest consumed by the renderer. There are no environment overrides for content semantics.

`site.base_path` is either empty or a canonical URL path with one leading slash, no trailing slash, and URL-safe segments. `assets_out` must be a non-empty URL-safe subdirectory of `public/`; its relative path becomes the browser-facing prefix for generated `theme.css` and the `assets/` content subtree.

## Commands

### `mbsite check`

Runs discovery, parsing, resolution, and validation without modifying generated output. It exits nonzero when any error exists.

### `mbsite build`

Runs the complete repository-local build:

1. Load and validate content and theme configuration.
2. Parse, resolve, and validate the complete content graph.
3. Generate TypeScript and the theme/content-asset tree into separate managed outputs.
4. Invoke the configured framework adapter build without a shell.
5. Verify that the configured static output directory exists.

`mbsite build --content-only` stops after generated content and theme output. It exists for local development integration; production builds use the complete command.

### `mbsite init [path]`

Creates the default site in an empty or Git-only directory. The scaffold includes content, configuration, a complete `mambo.theme.toml`, the framework adapter, package scripts, and a GitHub Pages workflow.

Initialization never recursively cleans an unknown directory. `--force` refreshes only paths recorded as scaffold-owned and preserves unknown files; it still refuses an arbitrary non-scaffold directory. Initialization does not access the network or install dependencies. Run the chosen package manager explicitly and commit its lockfile before using the generated deployment workflow.

### `mbsite deploy`

Runs a complete local build, verifies repository and GitHub configuration, pushes committed work, and starts the configured GitHub Pages workflow. It does not synchronize an external vault and does not silently commit uncommitted work.

When the current commit is already on the remote, deployment uses GitHub Actions `workflow_dispatch`. GitHub Pages can therefore rebuild and deploy the same commit; an empty commit is unnecessary. `--dry-run` performs the local build and reports the resolved push or workflow action without fetching, pushing, or dispatching external state. It requires an existing local remote-tracking branch; fetch once before a dry run.

### Planned: `mbsite inspect <target>`

This future command will explain a page or reference: source path, route, mount, metadata derivation, children, links, embeds, assets, and diagnostics. Targets may be source paths, routes, or wikilinks. It is not part of the current CLI.

### Planned: `mbsite watch`

Later command for local development. It watches the content root and configuration, rebuilds affected output, and reports diagnostics. The first release does not depend on it.

## Package scripts in a website

The site shell should provide predictable wrappers:

```json
{
  "scripts": {
    "content:check": "mbsite check",
    "content:build": "mbsite build --content-only",
    "predev": "mbsite build --content-only",
    "dev": "next dev",
    "build": "mbsite build",
    "mambosite:render": "next build"
  }
}
```

The exact package manager is configured using a supported enum. Renderer scripts are validated names and are executed directly through the package manager, never interpolated into a shell command.

## Next.js integration

The web shell uses one root route and one catch-all route instead of generating a `page.tsx` file for every Markdown page:

```text
src/app/page.tsx
src/app/[...slug]/page.tsx
```

The root route renders the configured entry page. The catch-all route:

1. Imports the generated manifest and page lookup.
2. Exports `generateStaticParams()` for every non-root, non-draft route.
3. Sets dynamic parameters to false.
4. Resolves `/` and every slug to a generated `PageRecord`.
5. Generates page metadata from the record.
6. Renders the page through the MamboSite React runtime and selected component registry.
7. Returns the normal not-found result for absent routes.

The site config uses static export:

```ts
import type { NextConfig } from "next";
import manifest from "./src/generated/mambo/manifest";

const nextConfig: NextConfig = {
  output: "export",
  basePath: manifest.site.basePath,
  trailingSlash: manifest.site.trailingSlash,
  images: { unoptimized: true },
};

export default nextConfig;
```

Next.js currently generates an `out/` directory for `output: "export"`. Features requiring a runtime server are outside MamboSite's deployment model. The official static-export guide is [Next.js: Static Exports](https://nextjs.org/docs/app/guides/static-exports).

Static export does not support the default request-time image optimizer. The initial runtime should use ordinary responsive images or an explicitly static-compatible image strategy. Content correctness must not depend on an external image service.

## Base path and URLs

The current Next adapter prepends the configured base path to internal links and image paths. It also uses `site.url`, when present, as Next.js metadata's base URL.

- A custom domain such as `https://projectmambo.org` normally uses an empty base path.
- A project Pages URL such as `https://projectmambo.github.io/MamboWiki` uses `/MamboWiki`.
- Internal route identity remains `/mambodot/commands/`; the runtime prepends the deployment base path when creating browser URLs.
- Canonical URL declarations, sitemap output, RSS, and richer Open Graph data are planned. Content-asset URLs are compiled now; hashing and media transformation remain future work.
- Content authors should not manually include the deployment base path in internal links.

`trailing_slash = true` is the preferred initial policy because directory-style routes map naturally to `route/index.html` on static hosts.

## GitHub Actions pipeline

The production workflow has separate build and deployment jobs.

Build job:

1. Check out the website repository.
2. Install the pinned Rust toolchain.
3. Restore Cargo caches safely.
4. Install or build the pinned MamboSite compiler.
5. Install the pinned Node.js and package-manager versions.
6. Run one full `mbsite build`, which validates content, regenerates managed output, and invokes the configured static renderer.
7. Verify that `out/` exists.
8. Upload `out/` with the GitHub Pages artifact action, whose artifact contract rejects symbolic and hard links.

Deployment job:

1. Depend on the successful build job.
2. Use the `github-pages` environment.
3. Request only `pages: write` and `id-token: write` in addition to read access.
4. Deploy the uploaded artifact with GitHub's Pages deployment action.

GitHub's official flow and current action versions are documented in [Using custom workflows with GitHub Pages](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages). Uploaded Pages artifacts must not contain symbolic or hard links.

Action major versions should be pinned and updated deliberately, preferably with automated dependency update proposals. Design documents should not make old action versions part of the MamboSite content contract.

The scaffold currently names the intended `v0.1.0` compiler tag and `0.1.0` npm packages. Those release coordinates are not published yet. Until they are, integration workflows must check out MamboSite explicitly and use workspace or file dependencies, as the MamboFolio migration does.

## Generated-file policy

Preferred policy:

- Commit the repository-local `docs/` tree because it is the public website content snapshot.
- Do not commit `src/generated/mambo/` or the generated `public/mambo/` tree.
- Rebuild generated data and theme CSS in local development and CI.
- Commit lockfiles for Rust and the website package manager.
- Pin the MamboSite compiler version used by a website.

A repository may temporarily commit generated output for migration, but CI must verify that regeneration produces no diff.

## Reproducibility

A production build must not require network access after dependencies are installed. The compiler does not fetch remote images, validate external links, read Git metadata for page dates, or insert the current time into semantic output.

Content data, routes, copied assets, and generated TypeScript remain deterministic. An ordinary output-producing `mbsite build` deliberately chooses a fresh collection-accent seed, so the accent selector order in generated theme CSS may change. Set `SOURCE_DATE_EPOCH` to an unsigned 64-bit integer in CI when the complete output must be byte-reproducible; the same settings and seed produce the same theme output, while an invalid value fails the build.

Build information may record compiler and schema versions in `build-info.ts`, but nondeterministic timestamps must not affect page modules, asset names, or route output.

## Failure behaviour

- Compiler errors leave previous generated output intact.
- Next.js build failure prevents artifact upload.
- Deployment never runs after a failed build.
- Warnings are shown in CI; a warning-escalation flag is planned.
- A successful deployment identifies the exact source commit; explicit compiler-version build metadata remains planned.

## Local preview

After installing dependencies and the `mbsite` command, the supported workflows are:

```bash
npm run dev
mbsite build
```

`npm run dev` uses `predev` to refresh generated content and theme output before starting Next.js. `mbsite build` performs the complete production pipeline and verifies `out/`. Later, `mbsite watch` may rebuild incrementally, but direct browser-side Markdown parsing should not be introduced for convenience.

## Package and schema versions

Generated data declares a schema version which `@mambosite/runtime` checks at startup. Website lockfiles will pin the independently versioned runtime, React registry, default theme, and framework adapter packages after their first publication. Different websites can then remain on different compatible package versions without copying MamboSite components.

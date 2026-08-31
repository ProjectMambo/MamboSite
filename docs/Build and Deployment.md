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
3. Generate TypeScript, theme CSS, and copied assets
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
│   └── _mounts/
│       ├── mambodot/
│       └── mambosite/
├── mambo.toml
└── src/
```

Before `mambosite check` or `mambosite build` runs, `docs/` must contain:

- The configured entry page.
- Every physical site page.
- Every mounted documentation tree.
- Every local link or embed target required by the site.
- Every referenced local asset.

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

[build]
mode = "production"
source_locations = false
search = true
```

Configuration precedence is deliberately small:

1. CLI flags for the current invocation.
2. `mambo.toml`.
3. documented defaults.

Environment variables may provide deployment-specific values such as the final base path, but they should not redefine content semantics.

## Commands

### `mambosite check`

Runs discovery, parsing, resolution, and validation without modifying generated output. It exits nonzero when any error exists.

### `mambosite build`

Runs the complete repository-local build:

1. Load and validate content and theme configuration.
2. Parse, resolve, and validate the complete content graph.
3. Generate TypeScript, theme CSS, and assets atomically.
4. Invoke the configured framework adapter build without a shell.
5. Verify that the configured static output directory exists.

`mambosite build --content-only` stops after generated content and theme output. It exists for local development integration; normal production builds use the complete command.

### `mambosite init [path]`

Creates the default site in an empty or Git-only directory. The scaffold includes content, configuration, theme settings, optional component overrides, the framework adapter, package scripts, and a GitHub Pages workflow.

Initialization never recursively cleans an unknown directory. Re-initialization may replace only files recorded as scaffold-owned, and refuses to destroy modified or unknown files. Dependency installation is an explicit option rather than an implicit network operation.

### `mambosite deploy`

Runs a complete local build, verifies repository and GitHub configuration, pushes committed work, and starts the configured GitHub Pages workflow. It does not synchronize an external vault and does not silently commit uncommitted work.

When the current commit is already on the remote, deployment uses GitHub Actions `workflow_dispatch`. GitHub Pages can therefore rebuild and deploy the same commit; an empty commit is unnecessary. `--dry-run` reports the resolved build, push, and workflow operations without mutating external state.

### `mambosite inspect <target>`

Explains a page or reference: source path, route, mount, metadata derivation, children, links, embeds, assets, and diagnostics. Targets may be source paths, routes, or wikilinks.

### `mambosite watch`

Later command for local development. It watches the content root and configuration, rebuilds affected output, and reports diagnostics. The first release does not depend on it.

## Package scripts in a website

The site shell should provide predictable wrappers:

```json
{
  "scripts": {
    "content:check": "mambosite check",
    "content:build": "mambosite build --content-only",
    "predev": "mambosite build --content-only",
    "dev": "next dev",
    "build": "mambosite build",
    "mambosite:render": "next build"
  }
}
```

The exact package manager is configured using a supported enum. Renderer scripts are validated names and are executed directly through the package manager, never interpolated into a shell command.

## Next.js integration

The web shell uses one optional catch-all route instead of generated `page.tsx` files:

```text
src/app/[[...slug]]/page.tsx
```

That route:

1. Imports the generated manifest and page lookup.
2. Exports `generateStaticParams()` from every production route.
3. Sets dynamic parameters to false.
4. Resolves `/` and every slug to a generated `PageRecord`.
5. Generates page metadata from the record.
6. Renders the page through the runtime and site component registry.
7. Returns the normal not-found result for absent routes.

The site config uses static export:

```ts
import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "export",
  trailingSlash: true,
};

export default nextConfig;
```

Next.js currently generates an `out/` directory for `output: "export"`. Features requiring a runtime server are outside MamboSite's deployment model. The official static-export guide is [Next.js: Static Exports](https://nextjs.org/docs/app/guides/static-exports).

Static export does not support the default request-time image optimizer. The initial runtime should use ordinary responsive images or an explicitly static-compatible image strategy. Content correctness must not depend on an external image service.

## Base path and URLs

MamboSite must generate URLs from the configured site URL and base path.

- A custom domain such as `https://projectmambo.org` normally uses an empty base path.
- A project Pages URL such as `https://projectmambo.github.io/MamboWiki` uses `/MamboWiki`.
- Internal route identity remains `/mambodot/commands/`; the runtime prepends the deployment base path when creating browser URLs.
- Canonical URLs, sitemap entries, RSS links, Open Graph metadata, and copied asset URLs use the same URL builder.
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
6. Run `mambosite check`.
7. Run the website build, which regenerates content and runs `next build`.
8. Verify that `out/` exists and contains no symbolic or hard links.
9. Upload `out/` as the GitHub Pages artifact.

Deployment job:

1. Depend on the successful build job.
2. Use the `github-pages` environment.
3. Request only `pages: write` and `id-token: write` in addition to read access.
4. Deploy the uploaded artifact with GitHub's Pages deployment action.

GitHub's official flow and current action versions are documented in [Using custom workflows with GitHub Pages](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages). That documentation also states that uploaded Pages artifacts must not contain symbolic or hard links, reinforcing the decision to emit materialized assets and static output.

Action major versions should be pinned and updated deliberately, preferably with automated dependency update proposals. Design documents should not make old action versions part of the MamboSite content contract.

## Generated-file policy

Preferred policy:

- Commit the repository-local `docs/` tree because it is the public website content snapshot.
- Do not commit `src/generated/mambo/` or `public/mambo/` generated copies.
- Rebuild generated data and assets in local development and CI.
- Commit lockfiles for Rust and the website package manager.
- Pin the MamboSite compiler version used by a website.

A repository may temporarily commit generated output for migration, but CI must verify that regeneration produces no diff.

## Reproducibility

A production build must not require network access after dependencies are installed. The compiler does not fetch remote images, validate external links, read Git metadata for page dates, or insert the current time into semantic output.

Build information may record compiler and schema versions in `build-info.ts`, but nondeterministic timestamps must not affect page modules, asset names, or route output.

## Failure behaviour

- Compiler errors leave previous generated output intact.
- Next.js build failure prevents artifact upload.
- Deployment never runs after a failed build.
- Warnings are shown in CI; selected warning codes may be elevated to errors.
- A successful deployment identifies the exact source commit and compiler version in workflow metadata.

## Local preview

The supported first workflow is:

```bash
mambosite build
npm run dev
```

The Next.js development server reads generated TypeScript. Later, `mambosite watch` may run alongside it, but direct runtime Markdown parsing should not be introduced for convenience.

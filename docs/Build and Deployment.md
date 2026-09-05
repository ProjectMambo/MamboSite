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

### One-time repository setup

Before the first deployment, configure the website repository on GitHub:

1. Commit and push the Pages workflow under `.github/workflows/` to the branch named by `[deploy].branch`.
2. Open **Settings → Pages**. Under **Build and deployment**, set **Source** to **GitHub Actions**. Do not select a branch publishing source; MamboSite uploads the finished `out/` artifact instead.
3. Match `mambo.toml` to the public URL:
   - A user or organisation site such as `https://example.github.io` uses that complete URL and `base_path = ""`.
   - A project site such as `https://example.github.io/my-site` uses that complete URL and `base_path = "/my-site"`.
   - A custom domain such as `https://example.com` uses that complete URL and normally `base_path = ""`.
4. Keep `workflow_dispatch` and the configured branch's `push` trigger in the workflow. Keep `contents: read` for checkout and grant the deployment job at least `pages: write` and `id-token: write`. The standard Pages actions use `GITHUB_TOKEN`; no personal access token or repository secret is required.
5. Keep the deployment job on the `github-pages` environment and set its URL from the deployment action's `page_url` output. GitHub creates this environment automatically if it does not exist. A deployment protection rule that permits only the default branch is recommended.
6. Push to the configured branch for the first run, then inspect **Actions** and **Settings → Pages** for the deployment result. GitHub's Pages setting selects Actions as the publishing method; it is not bound to one named workflow.
7. Enable **Enforce HTTPS** after it becomes available. For a custom domain, verify ownership when possible, add the domain under **Settings → Pages** before changing DNS, then use GitHub's current DNS values. With a custom Actions workflow, an existing repository `CNAME` file is ignored and is not required.

GitHub documents the current controls in [Configuring a publishing source](https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site), [Using custom workflows with GitHub Pages](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages), [Managing a custom domain](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site), and [Securing a Pages site with HTTPS](https://docs.github.com/en/pages/getting-started-with-github-pages/securing-your-github-pages-site-with-https).

Run `mbsite deploy --dry-run` locally before the first live deployment. The workflow must already exist on the remote branch for `workflow_dispatch` to start it. After the first push, `mbsite deploy` dispatches that workflow on the configured branch when local `HEAD` already equals the remote commit, so redeploying the same commit does not require an empty commit.

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

The scaffold substitutes the creating compiler's version into both its `vX.Y.Z` compiler checkout and four `@mambosite/*` package pins. GitHub source tags and npm package publication are separate; until the packages are published, integration workflows must check out MamboSite explicitly and use workspace or file dependencies, as MamboFolio and MamboWiki do.

## Generated-file policy

Preferred policy:

- Commit the repository-local `docs/` tree because it is the public website content snapshot.
- Do not commit `src/generated/mambo/` or the generated `public/mambo/` tree.
- Rebuild generated data and theme CSS in local development and CI.
- Commit lockfiles for Rust and the website package manager.
- Pin the MamboSite compiler version used by a website.

A repository may temporarily commit generated output for migration, but CI must verify that regeneration produces no diff.

## Reproducibility

A production build must not require network access after dependencies are installed. The compiler does not fetch remote images, validate external links, or read Git metadata for page dates. Each output-producing CLI build records one current Unix timestamp in the generated manifest so `::timestamp` can render the build instant; page modules, asset names, and routes do not depend on it.

Content data, routes, copied assets, and generated page modules remain deterministic. An ordinary output-producing `mbsite build` records the current build time in `manifest.ts` and deliberately chooses a fresh collection-accent seed, so those two outputs may change. Set `SOURCE_DATE_EPOCH` to a supported non-negative Unix timestamp when the complete output must be byte-reproducible; it fixes both the manifest timestamp and theme seed, while an invalid or out-of-range value fails the build.

Future build information may also record compiler and schema versions, but it must not affect page modules, asset names, or route output.

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

## Maintainer source releases

`mbsite release` is intentionally unsupported: source releases mutate MamboSite's own Git and GitHub state rather than a consuming website. After the version-preparation commit passes every repository gate, push `main`, create and inspect an annotated tag, then create an editable draft release from that verified tag:

```bash
git tag -a vX.Y.Z
git show vX.Y.Z
git push origin refs/tags/vX.Y.Z
gh release create vX.Y.Z --repo ProjectMambo/MamboSite --verify-tag --draft --fail-on-no-commits --title "MamboSite vX.Y.Z" --notes-from-tag
gh release view vX.Y.Z --repo ProjectMambo/MamboSite
gh release edit vX.Y.Z --repo ProjectMambo/MamboSite --notes-file release-notes.md
gh release edit vX.Y.Z --repo ProjectMambo/MamboSite --draft=false --latest
```

The annotated-tag editor supplies the first description. Review or replace it while the GitHub release is still a draft; `gh release edit --notes-file` also updates a mutable release later. Release notes should summarize user-visible changes, compatibility, validation, and any publication boundary such as workspace-local npm packages. `--verify-tag` prevents GitHub from silently creating a tag at another commit.

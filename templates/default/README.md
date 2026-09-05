# MamboSite

This website is built from Markdown in `docs/` with MamboSite.

## Setup

`mbsite init` creates this scaffold without accessing the network. Install
the released MamboSite packages, then commit the generated `package-lock.json`
so GitHub Pages can reproduce the build:

```bash
npm install
git add package-lock.json
git commit -m "build: lock site dependencies"
```

The generated scaffold pins the MamboSite compiler and package compatibility
version `__MAMBOSITE_VERSION__`. GitHub source releases and npm package
publication are separate; until the `@mambosite/*` packages are published,
use sibling-checkout `file:` dependencies as described in the MamboSite docs.

## Commands

```bash
npm run dev       # compile Markdown, then start the Next.js development server
npm run build     # compile content and theme, then export the complete site
npm run deploy    # build, push committed work, and trigger GitHub Pages
```

Edit `docs/index.md` to change the home page. The starter page at
`docs/getting-started.md` can be replaced with your own content. Add Markdown
files or `folder/index.md` files below `docs/` for more pages. Site
configuration lives in `mambo.toml`. Add `mambo.theme.toml` only when the
built-in MamboColour and MamboFont-backed theme needs site-specific overrides.

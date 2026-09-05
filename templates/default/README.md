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

The default scaffold pins MamboSite package version `0.1.1`. Package
publication and the matching compiler release tag are still pending, so this
template remains a repository-development preview rather than a released
bootstrap path.

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

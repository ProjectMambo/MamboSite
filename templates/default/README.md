# MamboSite

This website is built from Markdown in `docs/` with MamboSite.

## Setup

`mambosite init` creates this scaffold without accessing the network. Install
the released MamboSite packages, then commit the generated `package-lock.json`
so GitHub Pages can reproduce the build:

```bash
npm install
git add package-lock.json
git commit -m "build: lock site dependencies"
```

The default scaffold expects the `0.1.0` release of MamboSite and checks out the
matching compiler tag in its Pages workflow.

## Commands

```bash
npm run dev       # compile Markdown, then start the Next.js development server
npm run build     # compile content and theme, then export the complete site
npm run deploy    # build, push committed work, and trigger GitHub Pages
```

Edit `docs/index.md` to change the home page. The starter page at
`docs/getting-started.md` can be replaced with your own content. Add Markdown
files or `folder/index.md` files below `docs/` for more pages. Site
configuration lives in `mambo.toml`; design tokens live in
`mambo.theme.toml`.

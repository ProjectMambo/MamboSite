---
title: Theme and Components
description: Replaceable presentation system using MamboFolio as the initial visual reference.
order: 55
---

# Theme and Components

## Design status

MamboFolio is the initial visual reference for MamboSite, but it is not the permanent design specification. MamboFolio itself may be redesigned. The compiler and Markdown language must therefore depend on semantic component names and design tokens, never on its current React structure or Tailwind class strings.

The first runtime should reuse or reinterpret MamboFolio's strongest visual traits while cleaning up component boundaries.

## Traits worth carrying forward

The current MamboFolio establishes a recognizable Project Mambo style:

- MamboColour-backed semantic colour variables and multiple light/dark themes.
- MamboFont or a monospace fallback for a technical editorial character.
- A bounded, centered article column with generous responsive padding.
- Strong two-pixel borders and restrained surface layers.
- Square colour canvases or images as card headers.
- Responsive grid and list presentations for projects, posts, and galleries.
- Clear metadata chips, descriptions, dates, and external links.
- A sticky navigation bar with an accessible theme switch.
- A floating table-of-contents navigator for long documents.
- Small, consistent transitions rather than heavy animation.

These are defaults, not parser rules. A future theme may change spacing, typography, shape, navigation, cards, or motion without recompiling Markdown under a new language.

## Layered runtime structure

The TypeScript runtime should have five presentation layers:

```text
design tokens
    -> primitives
    -> content renderers
    -> directive components
    -> site shell and page layouts
```

### Design tokens

Tokens express purpose rather than a particular palette value:

```text
color.background
color.surface
color.border
color.foreground
color.foregroundMuted
color.brand
color.interactive
color.success
color.warning
color.danger
font.body
font.heading
space.*
radius.*
shadow.*
motion.*
contentWidth.*
```

MamboColour maps onto these tokens. Directive properties such as `tone="warning"` refer to semantic tokens, never `--wildfire` or a literal hexadecimal colour.

### Primitives

Initial primitives:

- `Text`
- `Link`
- `ButtonLink`
- `Divider`
- `Surface`
- `Stack`
- `Grid`
- `Media`
- `Tag`
- `Icon`

Primitives should expose small typed variants. Authored content cannot pass raw classes into them.

### Content renderers

Content renderers map normalized AST nodes to semantic HTML:

- Paragraph and inline formatting.
- Heading with compiler-generated ID.
- Lists and task lists.
- Code and syntax-highlighted code blocks.
- Tables.
- Block quotes and callouts.
- Images, audio, video, PDFs, and downloads.
- Footnotes and math.
- Links and embeds.

These renderers should work across every site theme.

### Directive components

The component registry maps directive nodes to runtime components:

```text
page           -> layout selection
hero           -> Hero
breadcrumbs    -> Breadcrumbs
meta           -> Metadata
toc            -> TableOfContents
children       -> ContentCollection
related        -> ContentCollection
backlinks      -> BacklinkList
gallery        -> Gallery
include        -> Embed/inline content renderer
button         -> ButtonLink
section        -> Section
columns        -> Columns
column         -> Column
```

`children view="grid"` may initially resemble MamboFolio's bordered square cards. It should still be implemented as a semantic collection component so a redesign can replace the entire card appearance.

### Site shell and layouts

The site repository owns:

- Header and primary navigation.
- Footer.
- Theme selection and persistence.
- Site metadata and icons.
- Page chrome.
- Layout implementations for `default`, `article`, `docs`, `project`, `collection`, `home`, and `gallery`.
- Optional search UI.

The shared runtime may provide defaults, but MamboFolio and MamboWiki may override layout/component implementations through a typed registry.

## Component override contract

The runtime should expose a registry rather than hardcoded imports:

```ts
interface MamboComponentRegistry {
  layouts: LayoutRegistry;
  directives: DirectiveRegistry;
  nodes: ContentNodeRegistry;
}
```

Each website starts with the default registry and replaces selected entries. Overrides receive the same validated node/prop types. They must not need access to raw Markdown or YAML.

This permits:

- MamboFolio to use expressive cards and portfolio layouts.
- MamboWiki to use a denser documentation sidebar and hierarchy tree.
- A future redesign to change presentation without migrating content.

## Initial page layouts

### `default`

Centered article, standard metadata and footer, optional floating TOC.

### `article`

Narrower reading width, publication metadata, strong typography, footnotes, and related posts.

### `docs`

Documentation navigation, breadcrumbs, content body, heading navigation, previous/next page controls, and backlinks when requested.

### `project`

Project summary, status/period, external links, cover/canvas, body, and related documentation.

### `collection`

Page introduction followed by one or more generated child/related collections.

### `home`

Wide composition with optional hero and multiple content sections. MamboFolio's current banner plus project/blog/gallery/contact progression is a reference, not a fixed sequence.

### `gallery`

Media-first layout with responsive grid or masonry presentation and accessible captions.

## Responsive behaviour

- Content must remain usable from narrow mobile screens through wide desktop screens.
- `columns` collapse at their declared breakpoint.
- Card grids choose safe responsive minimum widths; `columns=6` is a maximum intent, not a command to squeeze six unreadable cards onto mobile.
- Tables may scroll horizontally without widening the page.
- Navigation must remain keyboard accessible when wrapping or collapsing.
- The floating TOC must not cover content and should fall back to an inline or drawer presentation on small screens.

## Accessibility baseline

- Semantic HTML is preferred over role-heavy generic containers.
- Heading hierarchy comes from the compiler and must not be changed for visual size.
- Every interactive element is reachable and visible by keyboard.
- Focus indicators meet contrast requirements.
- Images require meaningful alt text where content-bearing.
- Decorative canvases and images are marked accordingly.
- Colour is not the only carrier of status.
- Light and dark theme tokens must satisfy readable contrast.
- Motion respects `prefers-reduced-motion`.
- Embedded documents expose their source and boundary accessibly.

## Client JavaScript policy

The default page body should render on the server during the static build. Client components are limited to features that genuinely require browser state:

- Theme switching and persistence.
- Collapsible mobile navigation.
- Optional floating TOC scroll tracking.
- Optional client-side search.
- Optional carousel/gallery interaction.

Cards, Markdown, navigation links, callouts, embeds, and ordinary child collections must work without hydration.

## Redesign rules

A style redesign may change:

- CSS framework.
- Token values and palette mapping.
- Typography and spacing.
- Card and canvas appearance.
- Header, footer, sidebar, and TOC interaction.
- Layout composition and breakpoints.
- Animation.

A style redesign must not require changing:

- Canonical Markdown.
- Mount definitions.
- Routes.
- Directive names or meanings.
- Generated page IDs.
- Rust parsing rules.

If a redesign reveals that a directive encodes appearance rather than intent, add a semantic capability or runtime default instead of adding CSS escape hatches to Markdown.

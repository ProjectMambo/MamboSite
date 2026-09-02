---
description: Replaceable presentation system using MamboFolio as the initial visual reference.
title: Theme and Components
order: 55
---

# Theme and Components

## Design status

MamboFolio is the initial visual reference for MamboSite, but it is not the permanent design specification. MamboFolio itself may be redesigned. The compiler and Markdown language must therefore depend on semantic component names and design tokens, never on its current React structure or Tailwind class strings.

The current default runtime reinterprets MamboFolio's strongest visual traits while keeping component boundaries replaceable.

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

## Site settings

Every site may provide `mambo.theme.toml`. It contains presentation settings only and overrides the built-in default recursively. Schema 1 accepts only `extends = "default"`; named third-party preset inheritance is not implemented.

```toml
schema = 1
id = "mambofolio"
extends = "default"
default_scheme = "dark"

[colors.dark]
background = "#181615"
text = "#faf7f2"
brand = "#d44b36"

[fonts]
body = "MamboFont, ui-monospace, monospace"
heading = "MamboFont, ui-monospace, monospace"

[breakpoints]
compact = 640
content = 900
wide = 1200

[widths]
reading = "48rem"
normal = "74rem"
sidebar = "15rem"

[layout.page_with_sidebar_columns]
base = "minmax(0, 1fr)"
content = "minmax(0, 1fr) var(--mambo-width-sidebar)"

[components.collection.max_columns]
base = 1
compact = 2
content = 6
```

MamboSite validates this file and generates `theme.ts` plus `theme.css`. Colours, fonts and font faces, type sizes, spacing, content widths, component dimensions, borders, shadows, motion, responsive layout templates, and component behavior are typed semantic tokens. The default component package requires that generated stylesheet and consumes only the `--mambo-*` contract for site-variable values.

CSS custom properties carry values such as colours and spacing. Breakpoint thresholds cannot use CSS variables in normal media queries, so MamboSite writes the configured breakpoint values as literal generated media rules. Complex structural redesigns remain component overrides rather than an attempt to encode arbitrary CSS in TOML.

## Layered runtime structure

The TypeScript runtime has five presentation layers:

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

The current public primitive registry contains `Link` and `Image`, the two framework-sensitive elements. Other markup stays inside typed node and directive components until a second implementation proves that another primitive boundary is useful. Authored content cannot pass raw classes.

### Content renderers

Content renderers map normalized AST nodes to semantic HTML:

- Paragraph and inline formatting.
- Heading with compiler-generated ID.
- Lists and task lists.
- Code blocks with language metadata for future highlighting.
- Tables.
- Block quotes and callouts.
- Images and note embeds.
- Footnotes and math.
- Links and embeds.

Audio/video/PDF classification and content-asset publication are planned. Raw HTML is displayed as code text rather than injected.

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

The default package currently renders direct child list/grid/card views and grid galleries. Tree/table child views, nested child depth, masonry/carousel galleries, and fragment includes show an explicit unsupported-mode message. A registry override may implement those contracts sooner.

### Site shell and layouts

MamboSite's default theme package owns:

- Header and primary navigation.
- Footer.
- Theme selection and persistence.
- Site metadata.
- Page chrome.
- Layout implementations for `default`, `article`, `docs`, `project`, `collection`, `home`, and `gallery`.
- Optional search UI when implemented.

A site repository supplies content data, theme settings, and an optional typed override registry. MamboFolio uses this default package during migration; MamboWiki is intended to start from it rather than copy component source.

## Component override contract

The runtime exposes a registry rather than hardcoded site imports:

```ts
interface MamboComponentRegistry {
  primitives: PrimitiveRegistry;
  nodes: NodeRegistry;
  directives: DirectiveRegistry;
  layouts: LayoutRegistry;
  shell: ShellRegistry;
  fallbacks: RegistryFallbacks;
}
```

Each website starts with the default registry and replaces selected entries. Overrides receive the same validated node/prop types and resolved content models. They must not need access to raw Markdown or YAML.

```ts
export const components = createRegistry(
  defaultRegistry,
  defineOverrides({
    directives: { children: ProjectCollection },
  }),
);
```

Registry maps are frozen and their TypeScript types require every node, directive, layout, shell, primitive, and fallback entry. `createRegistry` returns a new registry containing only the named replacements. A component can change markup or styling without changing parsing.

The generated content schema and npm packages have explicit versions. Sites will pin compatible package releases in their lockfiles, so one website can remain on an older component version while another upgrades. The first packages are still workspace-local and unpublished.

This permits:

- MamboFolio to use expressive cards and portfolio layouts.
- MamboWiki to use a denser documentation sidebar and hierarchy tree.
- A future redesign to change presentation without migrating content.

## Initial page layouts

### `default`

Centered page with generated title, body, parent link, and optional heading sidebar.

### `article`

The default page with a narrower reading width.

### `docs`

The default page with its heading sidebar enabled. Breadcrumbs, backlinks, and collections remain explicit body directives; hierarchy navigation and previous/next controls are not automatic yet.

### `project`

The default page tagged with the `project` layout for theme-specific styling.

### `collection`

A wider default page; child and related collections remain explicit directives.

### `home`

A wider default page whose composition comes from hero, section, collection, and column directives.

### `gallery`

A wider default page. The grid gallery directive works now; masonry and carousel behavior are planned.

## Responsive behaviour

- Content must remain usable from narrow mobile screens through wide desktop screens.
- `columns` collapse at their declared breakpoint.
- Card grids choose safe responsive minimum widths; `columns=6` is a maximum intent, not a command to squeeze six unreadable cards onto mobile.
- Tables may scroll horizontally without widening the page.
- Navigation must remain keyboard accessible when wrapping or collapsing.
- The floating TOC must not cover content and should fall back to an inline or drawer presentation on small screens.

Viewport thresholds never live in component CSS. Rust writes the configured compact, content, and wide values into literal media queries and emits finite selector rules for authored collection and column choices.

## Accessibility baseline

These are release requirements, not a claim that a complete automated accessibility audit has passed:

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

The page body renders during the static build. Current client code is limited to theme switching/persistence, the header's hide-on-scroll behavior, and its clock. Collapsible navigation, TOC tracking, search, and carousel interaction are planned.

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

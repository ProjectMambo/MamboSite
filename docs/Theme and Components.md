---
description: Replaceable presentation system using MamboFolio as the initial visual reference.
title: Theme and Components
order: 55
---

# Theme and Components

This document covers the default presentation contract and override boundary. Authors choosing page layouts should begin with [[Authoring Guide]]; theme implementers can use the complete contracts here.

## Design status

MamboFolio is the initial visual reference for MamboSite, but it is not the permanent design specification. MamboFolio itself may be redesigned. The compiler and Markdown language must therefore depend on semantic component names and design tokens, never on its current React structure or Tailwind class strings.

The current default runtime reinterprets MamboFolio's strongest visual traits while keeping component boundaries replaceable.

## Traits worth carrying forward

The current MamboFolio establishes a recognizable Project Mambo style:

- MamboColour-backed semantic colour variables and multiple light/dark themes.
- MamboFont or a monospace fallback for a technical editorial character.
- A bounded, centered article column with generous responsive padding.
- Strong two-pixel borders and restrained surface layers.
- Square controls, cards, tags, code blocks, and panels; every default radius token is zero.
- Square colour canvases or images as card headers.
- Responsive grid and list presentations for projects, posts, and galleries.
- Clear metadata chips, descriptions, dates, and external links.
- A sticky navigation bar with a vertically centered rectangular brand control and accessible theme switch.
- A conditional table-of-contents navigator for long documents.
- Small hover, active, and movement transitions rather than heavy animation.

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
brand = "#e05a45"
brand_hover = "#f08068"
brand_active = "#d96752"
on_brand = "#181615"
accents = ["#9cbaac", "#ffa775", "#c2cca8", "#8b9cbd", "#e08a4f", "#bf8087"]

[colors.light]
accents = ["#6b8c85", "#de8554", "#8fa382", "#4e687d", "#b86935", "#8c5258"]

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
gallery_image_max = "24rem"

[typography.body]
size = "1.125rem"
line_height = "1.72"
weight = 400
letter_spacing = "normal"

[typography.navigation.size]
base = "1.2rem"
compact = "1.25rem"

[dimensions]
control_min_height = "2.75rem"

[radii]
small = "0"
medium = "0"
large = "0"
pill = "0"

[layout.page_with_sidebar_columns]
base = "minmax(0, 1fr)"
content = "minmax(0, 1fr) var(--mambo-width-sidebar)"

[components.collection.max_columns]
base = 1
compact = 2
content = 2
wide = 6

[components.sidebar.mode]
base = "inline"
content = "sticky"
```

MamboSite validates this file and generates `theme.ts` plus `theme.css`. Colours, fonts and font faces, type sizes, spacing, content widths, component dimensions, borders, shadows, motion, responsive layout templates, and component behavior are typed semantic tokens. `brand`, `brand_hover`, and `brand_active` provide distinct resting, hover, and pressed states. The larger body and navigation styles, compact control height, square radii, and gallery media cap are defaults that a site may replace in the same settings file. The default component package requires the generated stylesheet and consumes only the `--mambo-*` contract for site-variable values.

CSS custom properties carry values such as colours and spacing. Breakpoint thresholds cannot use CSS variables in normal media queries, so MamboSite writes the configured breakpoint values as literal generated media rules. Complex structural redesigns remain component overrides rather than an attempt to encode arbitrary CSS in TOML.

## Default-theme entry and page data

The core compiler accepts JSON-compatible values beneath frontmatter `data`. The default theme recognizes these exact optional shapes:

```yaml
data:
  navigation:
    - label: MY SITE
      href: /
    - label: PROJECTS
      href: /projects/
  hero:
    quote: A short quotation.
    attribution: Its source
  footer:
    copyright: 2026 My Site
    links:
      - label: Source Code
        href: https://github.com/example/site
```

`navigation` and `footer` are read from the site entry. The first navigation item becomes the brand control; later items become the primary navigation. When no first item exists, the configured site title becomes the brand label. `footer.copyright` is rendered after a generated copyright symbol. `hero` is read from the current page when a `::hero` directive is present. Invalid or differently shaped values are ignored by the default theme rather than acquiring new semantics.

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
color.brandHover
color.brandActive
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

The compiler publishes content assets, while specialized audio/video/PDF classification and renderers remain planned. Raw HTML is displayed as code text rather than injected.

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

`children view="grid"` renders ordinary page-preview cards. `children view="cards" show=["title"]` renders the same child-page routes as a compact grid of button-like cards. This remains a semantic page collection, so it cannot represent arbitrary external destinations. A contact or action grid uses `columns` containing `button variant="card"` directives instead. The buttons remain links, fill their cells, and cycle through the same positional accent borders as content cards.

The default package currently renders direct child list/grid/card views and grid galleries. Tree/table child views, nested child depth, masonry/carousel galleries, and fragment includes show an explicit unsupported-mode message. A registry override may implement those contracts sooner.

## Collection accent assignment

Content cards and `button variant="card"` action cells draw accents from the paired `colors.dark.accents` and `colors.light.accents` arrays. These arrays are the complete configuration surface: both arrays must have the same length from 1 to 12, and entries may be any valid CSS colour.

When both schemes use at least two sRGB hexadecimal colours in `#RGB` or `#RRGGBB` form, MamboSite compares paired slots in OKLab. Two slots are similar when their Euclidean distance is less than `0.10` in either scheme. If a cycle exists that keeps every neighboring pair at or above that threshold, the generated grid guarantees dissimilar direct neighbors. A one-colour, non-hex, or mathematically incompatible palette remains valid and uses a seeded shuffle instead; no algorithm can guarantee perceptual separation for values it cannot measure or a palette without a valid cycle.

Each output-producing `mbsite build` chooses a fresh standard-library-backed seed and shuffles the selected order. `SOURCE_DATE_EPOCH=<unsigned-integer>` fixes that seed for reproducible builds. Because the number of possible orders is finite, separate unseeded builds may occasionally choose the same result.

For a grid with `N` accent slots, the generated CSS maps a cell at zero-based row and column to `order[(row + column) % N]`. When a safe OKLab cycle is available, direct neighbors above, below, left, and right therefore use dissimilar slots at every responsive column count. Dark and light schemes retain the same slot assignment and use their paired colour values. The mapping is compiled into CSS and needs no browser-side randomization.

### Site shell and layouts

MamboSite's default theme package owns:

- Header and primary navigation.
- A vertically centered rectangular brand link with configured brand, hover, and active colours.
- A compact stacked navigation menu below the configured breakpoint and inline navigation above it.
- Footer.
- Theme selection and persistence.
- A reusable CSS tooltip, used by the theme control instead of a browser `title` popup.
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

Centered page with generated title, body, optional heading sidebar, and Back links at both the start and end of every non-root page. An ordinary unmodified primary click uses native browser history when a history entry exists, restoring the prior route and its saved scroll position—including the collection card or next-page control that opened the page. The link still targets the route parent for direct entry, no JavaScript, or a modified click.

### `article`

The default page uses the configured reading width when it has no automatic TOC. When a TOC is present, the outer frame uses the normal width so the article retains a useful reading measure beside the TOC rail.

### `docs`

The default page with its heading sidebar enabled. Breadcrumbs, backlinks, and collections remain explicit body directives; hierarchy navigation and previous/next controls are not automatic yet.

### `project`

The default page tagged with the `project` layout for theme-specific styling.

### `collection`

A wider default page; child and related collections remain explicit directives.

### `home`

A wider default page whose composition comes from hero, section, collection, and column directives.

### `gallery`

A wider default page. Center-aligned gallery hero media is capped by `widths.gallery_image_max` instead of expanding across the full page. The grid gallery directive works now; masonry and carousel behavior are planned.

## Table of contents behavior

The default runtime derives an automatic TOC from level-two through level-four headings when `page.sidebar` is enabled. Pages without matching headings render neither an empty TOC nor an empty sidebar, so both headerless articles and structured articles use the same layout safely. A valid authored `::toc` renders at its Markdown position, honors its own `collapse` value, and suppresses the automatic copy.

Below the configured `content` breakpoint, the automatic TOC is a native `<details>` disclosure before the article and starts closed. At and above that breakpoint, it is an expanded sticky rail beside the article. The rail has a viewport-bounded height and its own scrolling area, and it follows the active entry when a long TOC cannot fit at once. It never overlays the article. Article pages expand their outer frame only when that rail exists; gallery pages retain their wide frame.

Once a matching section is reached, each visible TOC tracks exactly one current section and marks its link with `aria-current="location"`. The active entry is the last heading that has crossed a header-aware top threshold. At the bottom of the document, the final matching heading wins even when its section is too short to reach that threshold; continued scrolling therefore advances the TOC to the end although the page itself cannot move farther. All links still use the compiler's heading IDs and work as ordinary anchors without scroll tracking.

## Responsive behaviour

- Content must remain usable from narrow mobile screens through wide desktop screens.
- `columns` collapse at their declared breakpoint.
- Card grids choose safe responsive minimum widths; `columns=6` is a maximum intent, not a command to squeeze six unreadable cards onto mobile.
- Tables and code blocks scroll horizontally without widening the page; page copy, cards, controls, and long destinations wrap instead of forcing overflow.
- Compact navigation opens from a labelled button into a stacked menu; it closes after link activation or Escape and does not consume persistent page height while closed.
- The automatic TOC is a closed inline disclosure below the content breakpoint and a viewport-bounded sticky rail at and above it, so it remains usable without overlaying the article.
- Images remain constrained to their container, collection columns are capped at each configured breakpoint, and layout children use shrink-safe grid and flex sizing.

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

The page body renders during the static build. Current client code is limited to theme switching/persistence, the compact navigation disclosure, the header's hide-on-scroll behavior and clock, history-aware Back clicks, and TOC current-section tracking. Search and carousel interaction are planned.

Cards, Markdown, navigation links, callouts, embeds, ordinary child collections, and the route-parent Back fallback work without hydration. Hydrated same-page fragment links replace their current hash entry, so several TOC jumps still need only one Back activation to leave the page. Link, navigation, brand, theme, card, and button feedback uses native CSS transitions. The reduced-motion media query disables smooth scrolling and reduces transition durations, so this feedback does not require another client animation system.

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

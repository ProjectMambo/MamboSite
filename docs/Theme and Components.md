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

- MamboColour-backed semantic colour variables for light and dark schemes.
- Bundled MamboFont web faces with a monospace fallback for a technical editorial character.
- A bounded, centered article column with generous responsive padding.
- Strong two-pixel borders and restrained surface layers.
- Square controls, cards, tags, code blocks, and panels; every default radius token is zero.
- Square colour canvases or images as card headers.
- Responsive grid and list presentations for projects, posts, and galleries.
- Clear metadata chips, descriptions, dates, and external links.
- A sticky navigation bar with a vertically centered rectangular brand control and accessible theme switch.
- A conditional table-of-contents navigator for long documents.
- Restrained hover, active, page-entry, and header transitions rather than heavy animation.

These are defaults, not parser rules. A future theme may change spacing, typography, shape, navigation, cards, or motion without recompiling Markdown under a new language.

## Site settings

Every site may provide `mambo.theme.toml`. It contains presentation settings only and overrides the built-in default recursively; omit it when the default is sufficient. Schema 1 accepts only `extends = "default"`; named third-party preset inheritance is not implemented.

```toml
schema = 1
id = "mambofolio"
extends = "default"
default_scheme = "dark"

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

MamboSite validates this file and generates `theme.ts` plus `theme.css`. Colours, fonts and font faces, type sizes, spacing, content widths, component dimensions, borders, shadows, motion, responsive layout templates, and component behavior are typed semantic tokens. `brand`, `brand_hover`, and `brand_active` provide distinct resting, hover, and pressed states. The larger body and navigation styles, compact control height, square radii, and gallery media cap are defaults that a site may replace in the same settings file. The default component package imports its bundled MamboFont faces, requires the generated stylesheet, and consumes only the `--mambo-*` contract for site-variable values.

## Provider-backed default updates

MamboSite consumes other Project Mambo tools through two maintainer-only adapters:

```bash
npm run sync:theme
npm run sync:theme:check
```

The colour adapter calls `mbcolor <theme> css --out <dir>`, maps named MamboColour tokens into the checked-in Rust default model, and validates required tokens and contrast. The font adapter calls `mbfont compile 0.2.4 --format woff2 --out <dir>` with a fixed `SOURCE_DATE_EPOCH`, then refreshes four checked-in web fonts and their generated stylesheet. `MAMBOCOLOUR_BIN` and `MAMBOFONT_BIN` may select an alternate executable for local testing.

These adapters are the dependency boundary: provider output is reviewed and committed in MamboSite, while ordinary compiler, package, MamboFolio, and MamboWiki builds use only repository-local files. A provider update never silently changes a consumer build.

The current snapshot was reviewed with MamboColour commit `66f0c26d6d6462c54c023a4842e49dc6fa0b3c1c` and MamboFont commit `62f199e3bc49f921434ff0082947441dd0fde07c`; the bundled font filenames carry artifact version `0.2.4`.

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

`navigation` and `footer` are read from the site entry. The first navigation item becomes the brand control; later items become the primary navigation. When no first item exists, the configured site title becomes the brand label. `footer.copyright` is rendered after a generated copyright symbol. A top-level entry `:::footer` adds compiled Markdown and directives between that copyright text and the footer navigation; `::timestamp` uses this slot in MamboFolio and MamboWiki. `hero` is read from the current page when a `::hero` directive is present. Invalid or differently shaped values are ignored by the default theme rather than acquiring new semantics.

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

MamboColour maps onto these tokens by provider token name. Directive properties such as `tone="warning"` refer to semantic tokens, never `--wildfire` or a literal hexadecimal colour.

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

Each output-producing `mbsite build` chooses a fresh standard-library-backed seed and shuffles the palette slots. Within each collection or action grid, cards consume that shuffled order without replacement: every configured colour is used once before the order resets and repeats. There is no neighbor-distance constraint.

`SOURCE_DATE_EPOCH=<unsigned-integer>` fixes both the shuffle and the manifest build timestamp for reproducible builds. Because the number of possible orders is finite, separate unseeded builds may occasionally choose the same result. Dark and light schemes retain the same slot assignment and use their paired colour values. The mapping is compiled into CSS and needs no browser-side randomization.

### Site shell and layouts

MamboSite's default theme package owns:

- Header and primary navigation.
- A vertically centered rectangular brand link with configured brand, hover, and active colours.
- A compact stacked navigation menu below the configured breakpoint and inline navigation above it.
- Footer with an optional entry-authored Markdown/directive slot.
- Theme selection and persistence.
- A reusable tooltip, used by the theme control instead of a browser `title` popup. Mouse-click focus cannot leave it stuck after the pointer departs; intentional keyboard focus keeps it available.
- Site metadata.
- Page chrome.
- Layout implementations for `default`, `article`, `docs`, `project`, `collection`, `home`, and `gallery`.
- Optional search UI when implemented.

A site repository supplies content data, optional theme settings, and an optional typed override registry. MamboFolio and MamboWiki use this default package, including its MamboColour-backed model and MamboFont assets, rather than copying provider output or component source.

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

Once a matching section is reached, each visible TOC tracks exactly one current section and marks its link with `aria-current="location"`. The active entry is normally the last heading that has crossed a header-aware top threshold. Clicking an entry selects and reveals that exact TOC link throughout the fragment jump, including for a short section near the physical bottom that cannot reach the threshold. At the physical bottom, further downward wheel, touch, or keyboard scroll intent advances through any remaining short sections one entry at a time and keeps the TOC rail following the highlight. Upward intent immediately restores the geometry-derived entry. All links still use the compiler's heading IDs and work as ordinary anchors without scroll tracking.

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

The page body renders during the static build. Current client code is limited to theme switching/persistence, the compact navigation disclosure, the header's hide-on-scroll behavior and clock, history-aware Back clicks, and TOC current-section tracking. Header visibility samples scroll position once per animation frame and ignores tiny direction reversals, while the TOC changes its active-link markup only when the active heading changes. Build timestamps render statically and do not add another client timer. Search and carousel interaction are planned.

Cards, Markdown, navigation links, callouts, embeds, ordinary child collections, and the route-parent Back fallback work without hydration. Hydrated same-page fragment links replace their current hash entry, so several TOC jumps still need only one Back activation to leave the page. Native wheel, touch, and history scrolling remain browser-owned; fragment jumps use CSS smooth scrolling. Page entry uses an opacity-only animation, while link, navigation, brand, theme, card, and button feedback uses native CSS transitions. The reduced-motion media query disables smooth fragment scrolling and reduces animation and transition durations, so this feedback does not require another client animation system.

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

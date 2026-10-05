---
description: Replaceable presentation system using MamboFolio as the initial visual reference.
title: Theme and components
order: 55
---

# Theme and components

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

The file emitted by `mbsite init` is human-editable but deliberately omits `colors.dark.accents` and `colors.light.accents`. The built-in theme and its serialized/default scaffold form omit empty accent keys as well. Omission keeps both arrays provider-managed, allowing each build seed to select paired MamboColour values. Adding either `accents` key selects site-owned custom mode—even when the explicit values equal the current provider defaults—so explicit empty arrays and one-sided keys fail validation; custom mode requires both arrays with the same non-empty length.

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

## Provider boundaries

The Rust `mambosite-theme` crate directly depends on MamboColour Rust crate `0.2.0` at revision `39f0b4e45ce3bb7be8a3ecda8081d7f77c6948e0`, pinned in both `Cargo.toml` and `Cargo.lock`. Its small consumer adapter calls stable UI role methods for the semantic defaults and `random_seeded()` for paired card accents. MamboSite mixes its `u64` build seed with each accent slot before folding the mixed value into the provider API's `u32` seed domain, then passes the same slot seed to light and dark selection. MamboColour embeds its CSV palettes in the provider crate, so ordinary Rust builds compile against the API without another installed command, runtime palette files, or materialized provider values in MamboSite source. Updating MamboColour means changing the manifest revision, refreshing the lockfile, and running the Rust theme and workspace tests.

MamboFont remains a maintainer-time generated-asset dependency. Its adapter exposes these existing umbrella commands:

```bash
npm run sync:theme
npm run sync:theme:check
```

Both commands now concern only MamboFont. The adapter calls `mbfont compile 0.2.4 --format woff2 --out <dir>` with a fixed `SOURCE_DATE_EPOCH`, then refreshes four checked-in web fonts and their generated stylesheet. `MAMBOFONT_BIN` may select an alternate executable for local testing.

The font command must come from MamboFont revision `62f199e3bc49f921434ff0082947441dd0fde07c`. Check out and install that exact provider revision before a theme refresh; it emits the `MamboFont-<Style>_v0.2.4.woff2` contract consumed by the adapter. The current MamboFont pilot emits `MamboFontPilot-*` files and is intentionally incompatible. Follow the pinned checkout's own setup instructions instead of substituting the current pilot command.

The font adapter is the generated-asset boundary: provider output is reviewed and committed in MamboSite, while ordinary compiler, package, MamboFolio, and MamboWiki builds use the repository-local files. The MamboColour boundary is instead the exact Cargo dependency and its public Rust API.

The current MamboColour dependency is crate `0.2.0` at commit `39f0b4e45ce3bb7be8a3ecda8081d7f77c6948e0`. The current MamboFont snapshot was reviewed with commit `62f199e3bc49f921434ff0082947441dd0fde07c`; the bundled font filenames carry artifact version `0.2.4`.

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

MamboSite's Rust theme adapter maps stable MamboColour role methods such as `bg()`, `fg()`, `brand_hover()`, and `error()` onto these consumer-owned tokens. Directive properties such as `tone="warning"` refer to MamboSite semantic tokens, never a private palette name or literal hexadecimal colour.

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

`children view="list"` renders full-width page-preview cards. `children view="grid"` arranges those previews in a multi-column grid. `children view="cards" show=["title"]` renders the same child-page routes as a compact grid of button-like cards. These remain semantic page collections, so they cannot represent arbitrary external destinations. A contact or action grid uses `columns` containing `button variant="card"` directives instead. The buttons remain links, fill their cells, and cycle through the same positional accent top lines as content cards. List, grid, compact-card, and action-card presentations all place the accent on the block-start border, which is the top border in the default horizontal writing mode.

The default package currently renders direct child list/grid/card views and grid galleries. Tree/table child views, nested child depth, masonry/carousel galleries, and fragment includes show an explicit unsupported-mode message. A registry override may implement those contracts sooner.

## Collection accent assignment

Content cards and `button variant="card"` action cells draw accents from paired dark/light slots. Provider-managed defaults and site-owned custom arrays intentionally use different selection paths.

When both accent keys are omitted, the theme is in provider mode and each output-producing `mbsite build` chooses a fresh standard-library-backed `u64` build seed. MamboSite mixes that build seed with each of the six slots, converts each mixed value to the provider's `u32` seed domain, and calls MamboColour `random_seeded()` with the resulting slot seed for both light and dark schemes. Because the provider files have matching order, each slot remains paired across schemes while high build-seed bits still affect selection. Cards use the resulting six slots in order and repeat the cycle; provider selection does not promise six unique colours.

When either accent key is present, the theme is in site-owned custom mode. Both keys must be present, neither array may be empty, and the arrays must have the same length from 1 to 12; otherwise validation fails. Every entry may be any valid CSS colour. MamboSite preserves the configured arrays in the compiled model and applies its existing seeded shuffle only to card assignment. Within each collection or action grid, every custom slot is used once before the shuffled order repeats; light and dark use the same shuffled indices.

`SOURCE_DATE_EPOCH=<unsigned-integer>` fixes the provider selection or custom-array shuffle together with the manifest build timestamp for reproducible builds. Separate unseeded builds may occasionally produce the same finite result. The mapping is compiled into CSS and needs no browser-side randomization.

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

A site repository supplies content data, optional theme settings, and an optional typed override registry. The Rust compiler maps the pinned MamboColour API into generated theme values; the default npm package supplies the components and bundled MamboFont assets. MamboFolio and MamboWiki consume those MamboSite boundaries rather than copying provider output or component source.

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
- Light and dark theme tokens must satisfy readable contrast; selection, focus, success, warning, and danger roles are checked against both background and surface.
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

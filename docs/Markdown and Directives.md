---
description: Authoring syntax and component directive contract for MamboSite pages.
title: Markdown and Directives
order: 30
---

# Markdown and Directives

This is the normative schema-1 syntax reference. Start with [[Authoring Guide]] when choosing a page pattern, then use this document to verify exact properties and current renderer limits.

## Design principles

MamboSite documents should remain useful Markdown in editors including Obsidian, on GitHub, and as plain text. Custom syntax is reserved for features that Markdown cannot express: page layout, generated collections, site components, and controlled transclusion.

The language has three layers:

1. YAML frontmatter for identity, publication, routes, and mounts.
2. Markdown and supported Obsidian-compatible syntax for authored content.
3. MamboSite directives for visible components and presentation intent.

Directives describe semantics such as “render these children as cards.” They must not accept arbitrary JSX, JavaScript, CSS classes, or Tailwind utilities. This keeps content portable and lets the visual design change independently.

## Markdown dialect

The baseline is CommonMark with these GitHub Flavored Markdown features enabled:

- Tables.
- Strikethrough.
- Autolinks.
- Task-list items.
- Fenced code blocks.

The initial Obsidian-compatible extensions are:

- Wikilinks and aliases: `[[Page]]`, `[[Page|Label]]`.
- Heading links: `[[Page#Heading]]`.
- Block links: `[[Page#^block-id]]`.
- Note and asset embeds: `![[Target]]`.
- Callouts using `> [!TYPE]` syntax.
- Footnotes.
- Highlight using `==text==`.
- Inline and display math where supported by the renderer.
- Obsidian comments using `%% hidden text %%`.
- Block identifiers such as `^example-id`.

The following are outside the first release:

- MDX, JSX, and JavaScript expressions.
- Dataview queries.
- Obsidian Bases.
- Buttons or commands executed by Obsidian plugins.
- Canvas files.
- Execution of Templater or any other editor plugin during the build. Authoring tools must already have produced ordinary Markdown.
- Arbitrary raw HTML. It is disabled by default and may become an explicit trusted-site option later.

## Directive syntax

MamboSite uses `::` for leaf directives and a fence of three or more `:` characters for container directives. This avoids the Markdown list ambiguity created by a `-/command(...)` syntax and aligns container parsing with Comrak's block-directive extension.

### Leaf directive

A leaf directive renders one component at its exact position and has no Markdown children:

```md
::children{view="grid" columns=3 sort="order" direction="asc"}
```

### Container directive

A container directive wraps Markdown or other directives:

```md
:::section{width="wide" tone="subtle"}

## Featured projects

::children{view="cards" limit=3}

:::
```

### Grammar

```text
leaf-directive      = "::" name attributes? block-end
container-open      = fence name attributes? line-end
container-close     = matching-fence whitespace? line-end
fence               = ":" ":" ":" (":"*)
attributes          = "{" attribute* "}"
attribute           = name "=" value
value               = string | number | boolean | array
array               = "[" (value ("," value)*)? "]"
name                = lowercase-letter (lowercase-letter | digit | "-")*
```

Rules:

- A directive marker must be the first non-whitespace text on its line.
- Up to three leading spaces are accepted; four spaces make an indented code block.
- Leaf directives must occupy their own block. A one-line directive ends at the line ending; a multiline directive ends after the matching `}` and may contain nothing else.
- Container directives must close with a fence containing exactly as many `:` characters as their opening fence.
- Nested containers use a longer outer fence than their inner fence so their closing markers are unambiguous.
- Directive names and property names use lowercase kebab-case.
- Strings use double quotes and support `\"` and `\\` escapes.
- Numbers are finite decimal numbers.
- Arrays contain scalar values only; nested maps are not part of the body syntax.
- Whitespace separates attributes. Commas are used only inside arrays.
- Directives are never recognized inside inline code, fenced code, raw code blocks, or escaped text.
- `\::name{}` displays literal directive text without invoking it.
- Arbitrary expressions and string interpolation are forbidden.

The Rust parser must preserve the directive's source span and raw spelling for diagnostics.

## Full example

```md
---
title: Projects
description: Projects in the Project Mambo ecosystem.
order: 20
---

::page{layout="collection" width="wide"}

# Projects

Software, design systems, and experiments maintained under Project Mambo.

::meta{show=["description","tags"]}

::children{
  view="grid"
  columns=3
  sort="order"
  direction="asc"
  show=["cover","title","description"]
}

:::section{width="normal" tone="subtle"}

## Elsewhere

::button{label="Project Mambo on GitHub" href="https://github.com/ProjectMambo" variant="secondary" external=true}

:::
```

Multiline attributes are accepted only between the opening `{` and matching `}`. They remain attributes, not YAML. The closing `}` must occur before any Markdown body belonging to a container.

## Core directives

### `page`

Configures presentation for the current page without producing a visible node.

```md
::page{layout="docs" width="normal" sidebar=true}
```

Properties:

| Property | Values | Default |
|---|---|---|
| `layout` | `default`, `article`, `docs`, `project`, `collection`, `home`, `gallery` | `default` |
| `width` | `narrow`, `normal`, `wide`, `full` | `normal`; the default theme narrows or widens selected layouts as described in [[Theme and Components]] |
| `sidebar` | boolean | `true`; the `docs` layout always enables it |

It may appear at most once and must be the first body node other than comments or blank lines. It works on index and leaf pages.

### `hero`

Renders a prominent title area. Missing values are derived from frontmatter.

```md
::hero{image="assets/mambo.png" align="split" show-description=true}
```

Properties:

| Property | Values | Default |
|---|---|---|
| `image` | asset wikilink or path | frontmatter `cover` |
| `align` | `left`, `center`, `split` | `left` |
| `show-title` | boolean | `true` |
| `show-description` | boolean | `true` |
| `show-meta` | boolean | `false` |

### `breadcrumbs`

Renders route ancestry at its position.

```md
::breadcrumbs{home="Wiki" separator="/"}
```

Properties: `home` is the home-link label and defaults to `/`; `separator` defaults to `/`; `include-current` is boolean and defaults to `true`.

### `meta`

Renders selected page metadata rather than automatically dumping all frontmatter.

```md
::meta{show=["date","updated","tags","period"] style="inline"}
```

Properties:

- `show`: ordered array containing `title`, `description`, `date`, `updated`, `tags`, or keys beneath `data`; defaults to an empty array.
- `style`: `inline`, `stack`, or `table`; defaults to `stack`.
- `empty`: `hide` or `placeholder`; defaults to `hide`.

The runtime receives already validated values. It must not interpret arbitrary YAML.

### `timestamp`

Renders the instant recorded once for the current output build as a static, timezone-aware date and time.

```md
::timestamp{timezone="Asia/Singapore" label="Last built"}
```

| Property | Values | Default |
|---|---|---|
| `timezone` | IANA timezone identifier | `UTC` |
| `label` | string placed before the time | `Built` |

The generated manifest stores Unix epoch seconds. The default renderer uses the site language and the requested timezone to emit a semantic `<time>` element including date, time, seconds, and zone. An invalid timezone stops static rendering with an error. `SOURCE_DATE_EPOCH` supplies the recorded instant when set; otherwise the CLI records the current build time.

### `footer`

Provides entry-authored content for the site footer while remaining hidden at its source position in the page body.

```md
:::footer

::timestamp{timezone="Asia/Singapore" label="Last built"}

:::
```

`footer` accepts no properties. It may appear once, must be a top-level container on the configured entry page, and may contain ordinary Markdown or other valid directives. The default theme renders its children between the legacy `data.footer.copyright` text and footer navigation. Headings inside it are excluded from the page heading index and automatic table of contents.

### `toc`

Renders a table of contents from the compiler's heading index.

```md
::toc{min-depth=2 max-depth=4 ordered=false}
```

| Property | Values | Default |
|---|---|---|
| `min-depth` | integer from 1 to 6 | `2` |
| `max-depth` | integer from 1 to 6 | `4` |
| `ordered` | boolean | `true` |
| `title` | string | `On this page` |
| `collapse` | boolean | `false` |

The minimum may not exceed the maximum. The default component tracks the current heading and sets `aria-current="location"`; see [[Theme and Components]] for automatic desktop and mobile placement.

### `children`

Renders descendant pages of the current index page or another index page.

```md
::children{source="/project/" view="grid" columns=3 depth=1 sort="order" direction="asc" show=["cover","title","description"]}
```

Properties:

| Property | Values | Default |
|---|---|---|
| `source` | index-page route or note reference | current page |
| `view` | `list`, `grid`, `cards`, `tree`, `table`, `hidden` | `list` |
| `depth` | positive integer or `-1` for all | `1` |
| `sort` | `order`, `title`, `date`, `updated`, `path` | `order` |
| `direction` | `asc`, `desc` | depends on sort |
| `columns` | integer from 1 to 6 | `3` |
| `limit` | positive integer | unlimited |
| `show` | array of preview fields | theme default |
| `include-unlisted` | boolean | `false` |
| `empty` | `hide` or `message` | `hide` |

`children` is valid only on `index.md` in the first release. Without `source`, it uses the current page's route children. With `source`, the current runtime resolves a route or note-style reference to another compiled index page and uses that page's children; the source page itself is not rendered. Compiler-authoritative directive-source resolution is not implemented yet, so authors must verify these references in the rendered site as well as with `mbsite check`. Mounted and physical children otherwise behave consistently. `view="hidden"` declares that child routes exist without displaying them at this point.

### `related`

Renders pages related through tags or explicit links.

```md
::related{by="tags" view="cards" limit=4}
```

Properties: `by` (`tags`, `links`, or `both`, default `tags`), `view` (default `cards`; the default theme supports `list`, `grid`, and `cards`), `limit` (default `4`), `show` (default empty), and `include-unlisted` (default `false`). Ranking is deterministic and is calculated by the runtime from compiler-produced graph data.

### `backlinks`

Renders pages that link to the current page.

```md
::backlinks{view="list" limit=10}
```

Properties: `view` (`list` or `cards`, default `list`), `limit` (default unlimited), `show` (default empty), and `empty` (message string, default `Nothing links here yet.`).

### `gallery`

Renders a set of media assets or child pages.

```md
::gallery{source="children" view="grid" columns=3 fit="cover"}
```

Properties:

- `source`: `children`, or a route/note reference to a compiled index page; defaults to `children`.
- `view`: `grid`, `masonry`, or `carousel`; defaults to `grid`.
- `columns`: 1 through 6; defaults to `3`.
- `fit`: `cover`, `contain`, or `natural`; defaults to `cover`.
- `captions`: boolean; defaults to `true`.

The default renderer currently implements only `grid` over page children. `page-embeds`, arbitrary folders without an index page, masonry, and carousel behavior are planned.

### `include`

Provides explicit control over note transclusion. Plain `![[Note]]` remains the convenient default.

```md
::include{source="[[MamboDot]]" mode="inline" headings="shift" show-title=false}
```

Properties:

- `source`: required note or route reference.
- `mode`: `embed` or `inline`; defaults to `embed`.
- `headings`: `shift`, `keep`, or `strip-title`; defaults to `keep`.
- `show-title`: boolean; defaults to `true`.
- `show-source`: boolean; defaults to `false`.

The current runtime resolves whole-page sources from compiled content. Fragment sources produce an explicit unsupported message, and compiler-authoritative directive edges remain planned. Remote URLs are not supported by `include`.

### `button`

Renders a themed link while retaining link semantics.

```md
::button{label="Source code" href="https://github.com/ProjectMambo/MamboSite" variant="primary" external=true}
```

`label` and `href` are required. `variant` accepts `primary`, `secondary`, `quiet`, or `card` and defaults to `primary`; `external` defaults to `false`; `icon` is optional. `card` is intended for action/contact grids while retaining link semantics. An `assets/...` href uses the compiled content-asset namespace; other internal targets follow the current runtime resolution boundary described below.

### `section`

Container for grouping ordinary Markdown with layout intent.

```md
:::section{width="wide" tone="subtle" align="left"}

Markdown content.

:::
```

Properties:

- `width`: `narrow`, `normal`, `wide`, or `full`; defaults to `normal`.
- `tone`: `plain`, `subtle`, `brand`, `success`, `warning`, or `danger`; defaults to `plain`.
- `align`: `left`, `center`, or `right`; defaults to `left`.
- `id`: an explicit unique fragment identifier.

`tone` is semantic. It does not name a fixed colour.

### `columns` and `column`

Containers for simple responsive groups:

```md
::::columns{count=2 gap="normal" collapse-at="md"}

:::column

Left content.

:::

:::column

Right content.

:::

::::
```

`columns` requires `count` from 2 to 4. `gap` accepts `small`, `normal`, or `large` and defaults to `normal`; `collapse-at` accepts `sm`, `md`, `lg`, or `never` and defaults to `md`. `count` is the number of grid tracks, not a limit on items: the container may have more direct `column` children and wraps them into later rows, but it must have at least `count`. All direct directive children must be `column`; `column` accepts no properties.

## Current default-renderer coverage

The schema-1 parser and validator recognize the contracts above, but the initial default React theme intentionally reports an unsupported-mode message instead of silently approximating semantics it does not yet implement.

Current limitations are:

- `children` renders direct children with `list`, `grid`, `cards`, or `hidden`; recursive depth and the `tree` and `table` views remain pending.
- `gallery` renders the grid view from children of the current or referenced index page; `page-embeds`, arbitrary folders, masonry, and carousel behavior remain pending.
- Whole-page note embeds render through compiler-resolved graph edges. Heading or block fragment transclusion and structurally spliced inline includes remain pending.
- `children`, `gallery`, and `include` source properties currently resolve through the runtime content store rather than compiler-authored directive edges. Buttons support safe web URLs and ordinary site routes. Compiler-authoritative resolution of note-style or source-relative directive targets is planned for a later generated-schema revision.

These are renderer boundaries, not permission for themes to reinterpret the authored properties. A site override may implement a pending view through the typed component registry while keeping the same directive contract.

## Component registry

The Rust compiler and TypeScript runtime share a versioned registry defining:

- Directive name.
- Leaf or container form.
- Allowed contexts.
- Property names, types, defaults, and enum values.
- Whether Markdown children are allowed.
- Output component node name and schema version.

The compiler validates directives before generation. The runtime must never receive unknown properties and should still render an explicit unsupported-component message when compiler/runtime schema versions do not match.

The registry is a contract, not a style implementation. A `children` grid may be redesigned without changing Markdown as long as the semantic properties remain supported.

## Errors and forward compatibility

- Unknown directive: error.
- Unknown property: error with closest-name suggestion.
- Duplicate property: error.
- Missing required property: error.
- Invalid value or context: error.
- Deprecated directive/property: warning for one compatibility window, then error in the next schema version.
- A site may opt into future syntax only by increasing its declared schema version.

Strict failure is intentional. Rendering custom syntax as accidental text would hide broken pages.

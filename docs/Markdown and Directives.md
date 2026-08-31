---
description: Authoring syntax and component directive contract for MamboSite pages.
title: Markdown and Directives
order: 30
---

# Markdown and Directives

## Implementation checkpoint

The compiler lowers CommonMark/GFM and the enabled Comrak extensions into a MamboSite-owned parser tree with source spans. Schema-1 leaf and container directives are tokenized into typed properties and validated against the core directive registry, including defaults, required properties, leaf/container form, page context, and `columns`/`column` nesting. Obsidian-style comments, note embeds, and block IDs are lowered, and Markdown links, wikilinks, aliases, heading/block fragments, backlinks, and the embed dependency graph are resolved and validated.

This is a deliberately bounded Obsidian-compatible subset, not full Obsidian support. Directives still remain semantic parser nodes rather than renderer components, and note embeds are resolved references rather than expanded/transcluded AST fragments. Reference-bearing directive properties such as `include.source`, `button.href`, `hero.image`, and gallery sources are type-checked by the registry but do not yet enter the reference/asset graph. Structural component lowering and transclusion, directive-property resolution, local asset processing (including asset embeds), production filtering of draft pages, navigation/search derivation, runtime rendering, and deployment remain to be implemented.

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

The initial Obsidian-compatible syntax contract is:

- Wikilinks and aliases: `[[Page]]`, `[[Page|Label]]`.
- Heading links: `[[Page#Heading]]`.
- Block links: `[[Page#^block-id]]`.
- Note embeds: `![[Target]]`. Asset-shaped embeds use the same syntax, but local asset resolution and copying are not implemented in the current compiler slice.
- Callouts using `> [!TYPE]` syntax. The current parser retains Comrak's GitHub-style alert subset; the broader mapping is still planned.
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
| `width` | `narrow`, `normal`, `wide`, `full` | layout-dependent |
| `sidebar` | boolean | layout-dependent |

It may appear at most once and must be the first body node other than comments or blank lines. It works on index and leaf pages.

### `hero`

Renders a prominent title area. Missing values are derived from frontmatter.

```md
::hero{image="[[Attachments/mambo.png]]" align="split" show-description=true}
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

Properties: `home`, `separator`, and `include-current`. Defaults are theme-controlled, `/`, and `true`.

### `meta`

Renders selected page metadata rather than automatically dumping all frontmatter.

```md
::meta{show=["date","updated","tags","period"] style="inline"}
```

Properties:

- `show`: ordered array of core fields or keys beneath `data`.
- `style`: `inline`, `stack`, or `table`.
- `empty`: `hide` or `placeholder`.

The runtime receives already validated values. It must not interpret arbitrary YAML.

### `toc`

Renders a table of contents from the compiler's heading index.

```md
::toc{min-depth=2 max-depth=4 ordered=false}
```

Properties: `min-depth`, `max-depth`, `ordered`, `title`, and `collapse`. Heading depth must be between 1 and 6 and the minimum may not exceed the maximum.

### `children`

Renders descendant pages of the current index page.

```md
::children{view="grid" columns=3 depth=1 sort="order" direction="asc" show=["cover","title","description"]}
```

Properties:

| Property | Values | Default |
|---|---|---|
| `view` | `list`, `grid`, `cards`, `tree`, `table`, `hidden` | `list` |
| `depth` | positive integer or `-1` for all | `1` |
| `sort` | `order`, `title`, `date`, `updated`, `path` | `order` |
| `direction` | `asc`, `desc` | depends on sort |
| `columns` | integer from 1 to 6 | theme-responsive default |
| `limit` | positive integer | unlimited |
| `show` | array of preview fields | theme default |
| `include-unlisted` | boolean | `false` |
| `empty` | `hide` or `message` | `hide` |

`children` is valid only on `index.md` in the first release. It uses route children, so mounted and physical children behave consistently. `view="hidden"` declares that child routes exist without displaying them at this point.

### `related`

Renders pages related through tags or explicit links.

```md
::related{by="tags" view="cards" limit=4}
```

Properties: `by` (`tags`, `links`, or `both`), `view`, `limit`, `show`, and `include-unlisted`. Ranking must be deterministic and is calculated by Rust.

### `backlinks`

Renders pages that link to the current page.

```md
::backlinks{view="list" limit=10}
```

Properties: `view` (`list` or `cards`), `limit`, `show`, and `empty`.

### `gallery`

Renders a set of media assets or child pages.

```md
::gallery{source="children" view="grid" columns=3 fit="cover"}
```

Properties:

- `source`: `children`, `page-embeds`, or a logical folder path.
- `view`: `grid`, `masonry`, or `carousel`.
- `columns`: 1 through 6.
- `fit`: `cover`, `contain`, or `natural`.
- `captions`: boolean.

`carousel` may require a small client component; the other modes should render without client JavaScript.

### `include`

Provides explicit control over note transclusion. Plain `![[Note]]` remains the convenient default.

```md
::include{source="[[MamboDot#Installation]]" mode="inline" headings="shift" show-title=false}
```

Properties:

- `source`: a note, heading, or block wikilink.
- `mode`: `embed` or `inline`.
- `headings`: `shift`, `keep`, or `strip-title`.
- `show-title`: boolean.
- `show-source`: boolean.

The schema-1 contract requires the source to resolve during compilation and forbids remote URLs. The current registry validates its shape, while semantic resolution and participation in the embed graph remain part of the next lowering pass.

### `button`

Renders a themed link while retaining link semantics.

```md
::button{label="Source code" href="https://github.com/ProjectMambo/MamboSite" variant="primary" external=true}
```

Properties: `label`, `href`, `variant` (`primary`, `secondary`, `quiet`), `external`, and optional `icon`. The schema-1 semantic pass will resolve an internal `href` with the same rules as Markdown links; the current registry does not yet perform that reference/scheme check.

### `section`

Container for grouping ordinary Markdown with layout intent.

```md
:::section{width="wide" tone="subtle" align="left"}

Markdown content.

:::
```

Properties:

- `width`: `narrow`, `normal`, `wide`, or `full`.
- `tone`: `plain`, `subtle`, `brand`, `success`, `warning`, or `danger`.
- `align`: `left`, `center`, or `right`.
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

`columns` accepts `count` from 2 to 4, `gap` (`small`, `normal`, `large`), and `collapse-at` (`sm`, `md`, `lg`, `never`). Its direct directive children must be `column`, and their count must match `count`.

## Component registry

The Rust compiler has a schema-1 directive registry defining:

- Directive name.
- Leaf or container form.
- Allowed contexts.
- Property names, types, defaults, and enum values.
- Whether Markdown children are allowed.
- The normalized semantic properties that a future renderer component receives.

The compiler validates directives before generation and does not pass unknown properties onward. Mirroring this contract in the TypeScript renderer registry, lowering directives to renderable component nodes, and handling compiler/runtime schema mismatches remain presentation-layer work.

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

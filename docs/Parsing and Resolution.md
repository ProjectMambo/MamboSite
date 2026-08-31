---
description: Rust parsing pipeline and rules for links, embeds, headings, and assets.
title: Parsing and Resolution
order: 40
---

# Parsing and Resolution

## Input guarantees

Every source file must be valid UTF-8. An optional UTF-8 byte-order mark is accepted and removed. CRLF and CR line endings normalize internally to LF while source diagnostics retain correct line and column positions.

The compiler reads source without modifying it. Parsing and build operations must never rewrite source Markdown or mutate the configured content root.

## Content-root path model

MamboSite receives a repository-local content root, normally `docs/`, and a content-root-relative site entry, normally `index.md`, from `mambo.toml`. These configured paths define the complete namespace used by parsing and resolution.

Every source receives a logical path relative to the content root, normalized to forward slashes. Logical paths never contain an absolute prefix, `.` segments, escaping `..` segments, host-specific separators, or authoring-tool locations. Absolute host paths may appear only in internal I/O state; user diagnostics, generated TypeScript, and stable page identifiers use logical paths.

Ordinary discovery skips `_mounts/`, `_info.md`, `README.md`, and the other exclusions defined in [[Content Model]]. Each mount declared in the configured entry creates a second, explicit traversal beginning at a repository-local path such as `_mounts/mambodot/index.md`. This makes the selected subtree reachable without turning `_mounts` into a route segment or exposing unrelated stored projects.

Every mount source must remain within the content root after lexical normalization and filesystem canonicalization. The same containment rule applies to local note links, embeds, and assets. The compiler must reject absolute paths, root-escaping traversal, special files, and symlinks that escape the content root.

MamboSite does not discover an Obsidian vault or synchronize authoring files. Project Mambo's optional materialization workflow is described in [[Documentation Sync]].

## Parsing stages

### 1. Source scan

The scanner records the logical source path, byte length, content hash, and line-start offsets. It identifies regions that must be protected from custom syntax processing:

- Frontmatter.
- Fenced code blocks.
- Indented code blocks.
- Inline code spans.
- Escaped directive markers.

Custom syntax must not be implemented as global regular-expression replacement.

### 2. Frontmatter split

Frontmatter is recognized only when the first non-BOM line is exactly `---`. The closing delimiter must also be exactly `---` on its own line. A missing closing delimiter is an error.

YAML is deserialized into a generic value first and then validated into MamboSite fields. YAML aliases, tagged values, and executable/custom types should be rejected. Values passed into `data` must be representable as JSON.

### 3. Directive tokenization

Leaf directives are MamboSite-specific and must be tokenized with source spans before general Markdown lowering. Container directives may use Comrak's block-directive nodes, with MamboSite parsing the node's info string into a name and typed attributes.

The scanner must recognize balanced quotes, arrays, and braces. A malformed directive is a syntax error even if CommonMark could otherwise treat it as text.

### 4. Markdown AST

Comrak parses the Markdown body with explicitly selected extensions. The adapter then lowers Comrak's arena-backed nodes into MamboSite's owned AST. No Comrak node or lifetime may escape the adapter.

The owned AST retains:

- Node kind.
- Child order.
- Source file and byte/line span.
- Raw destination for unresolved links and embeds.
- Authored heading level.
- Code-block language and metadata.
- Directive name and raw properties.

### 5. Dialect lowering

When Obsidian compatibility is enabled, the adapter recognizes syntax not fully represented by the base parser, including `![[...]]`, heading/block fragments, comments, aliases, and block IDs. It converts them into explicit unresolved nodes rather than rendered HTML. This is a Markdown dialect feature; it does not give the compiler knowledge of a vault, `.obsidian/`, or an Obsidian installation.

### 6. Semantic resolution

Resolution occurs only after every relevant file is indexed. This allows forward links, aliases, backlinks, route collision checks, and cycle detection to work across the entire site.

### 7. Validation and derivation

The compiler validates nodes and derives routes, headings, summaries, child relationships, link targets, embed dependency order, asset destinations, navigation, and search text.

## Internal node model

The Rust intermediate representation should distinguish block, inline, and generated/component nodes. Representative nodes include:

```text
Document
Paragraph
Heading
Text
Emphasis
Strong
Delete
Highlight
InlineCode
CodeBlock
Link
Image
List
ListItem
BlockQuote
Callout
Table
ThematicBreak
FootnoteDefinition
FootnoteReference
Math
Directive
Embed
Asset
```

Resolution transforms unresolved nodes rather than replacing them with strings. For example, `WikiLink { raw_target }` becomes `Link { page_id, route, fragment }`, and an asset embed becomes `Image { asset_id, public_url, dimensions }`.

## Wikilinks

Supported forms:

```md
[[Page]]
[[Page|Visible label]]
[[Page#Heading]]
[[Page#Heading|Visible label]]
[[Page#^block-id]]
```

Resolution order for a note target:

1. Exact logical path relative to the source note's directory.
2. Exact content-root-relative logical path.
3. Exact normalized path without `.md`.
4. Unique filename/stem match among pages admitted by ordinary or explicit mount discovery.
5. Unique frontmatter alias match.

The first stage producing more than one candidate is ambiguous and fails. Case-insensitive fallback should be a warning-producing compatibility option, not the default.

After finding the source page, the resolver maps it to its route in the active site. A canonical document may receive a different route in different sites because mounts are site-local.

A link to a file present in the content root but not admitted to the active site's route graph is an error unless explicitly marked as a source-only reference in a future version. Merely storing a project beneath `_mounts/` does not make it linkable; an entry-page mount must make that subtree reachable.

## Standard Markdown links

Standard links remain supported:

```md
[Commands](Commands.md)
[Section](Commands.md#usage)
[External site](https://example.com)
```

Relative `.md` links resolve through the content graph and are rewritten to site routes. Root-relative links are interpreted relative to the site root. Absolute HTTP(S), `mailto`, and other allowed schemes remain external.

Unsafe schemes such as `javascript:` and malformed control-character URLs are errors. External links are not fetched or validated during a normal build.

## Heading identifiers

Heading IDs are generated in Rust so the table of contents, links, embeds, and rendered HTML agree without client-side repair.

Rules:

1. An explicit supported heading attribute wins when unique.
2. Otherwise use normalized plain heading text.
3. Remove formatting while retaining its visible text.
4. Lowercase ASCII and normalize whitespace/punctuation to `-`.
5. Preserve meaningful non-ASCII characters.
6. If empty, use `section`.
7. Add `-2`, `-3`, and so on for duplicates in document order.

The generated heading index contains the ID, plain text, authored depth, source span, and parent heading relationship. The runtime must use these IDs directly and must not generate random fallbacks.

## Block identifiers

An Obsidian block ID such as `^install-command` attaches to the immediately preceding block according to Obsidian-style placement rules. The marker itself is removed from visible output.

Block IDs must be unique within a page and use ASCII letters, digits, and `-`. Invalid or duplicate IDs are errors. A block link resolves to the containing page plus the stable block ID.

## Embeds and transclusion

Supported note forms:

```md
![[Page]]
![[Page#Heading]]
![[Page#^block-id]]
```

An embed is never implemented by copying a Markdown string into another string or adding whitespace indentation. Both operations can change list, code-block, and heading semantics. Instead, the compiler creates an embed node referring to a resolved AST fragment.

### Default embed mode

Plain `![[Page]]` uses `mode="embed"`:

- Render as a visibly bounded embedded article/section determined by the theme.
- Preserve a source-page reference.
- Keep source provenance on all nested nodes.
- Prefix generated DOM IDs with an embed-instance identifier to prevent collisions.
- Do not merge the embedded headings into the host page's main table of contents by default.

This is a semantic component boundary. The current theme may indent or border it, but physical Markdown indentation is never introduced.

### Inline mode

The `include` directive can request `mode="inline"`:

- Splice the resolved block nodes into the host flow.
- Include them in the host table of contents.
- Rewrite local fragment links to the embedded instance.
- Shift heading levels relative to the surrounding heading when `headings="shift"`.

Heading shifting is structural. If an include appears beneath a level-two heading and the embedded fragment begins at level one, its first heading becomes level three and descendants shift by the same amount, capped at level six. Overflow is an error rather than silently flattening hierarchy.

`headings="strip-title"` removes only the embedded document's initial H1 when it represents the document title. `headings="keep"` preserves authored levels and may produce a warning when hierarchy is broken.

### Cycles and limits

The resolver builds a directed embed graph and checks it before expansion. `A -> B -> A` and longer cycles are errors showing the complete chain. A configurable maximum depth, default 16, protects against pathological acyclic expansion.

## Images and local assets

Supported examples:

```md
![Alternative text](../Attachments/mambo.png)
![[Attachments/mambo.png]]
![[Attachments/mambo.png|640]]
![[Attachments/mambo.png|640x360]]
```

Asset lookup order mirrors note lookup but uses the exact filename including extension. When Obsidian compatibility is enabled, a unique-basename fallback may be used for attachment-style references; ambiguity is an error.

For images:

- Standard Markdown alt text is preserved.
- Obsidian embeds derive fallback alt text from the filename; authors should use standard Markdown where meaningful alternative text matters.
- A numeric pipe option is requested display width.
- `widthxheight` supplies requested display dimensions.
- Intrinsic dimensions should be read during compilation for supported raster formats.
- SVG is treated as an image but must be copied safely and not injected as raw markup by default.

Other local assets map by media type:

- Audio becomes an audio node.
- Video becomes a video node.
- PDF becomes a document/embed node according to runtime capability.
- Unknown files become downloadable-file nodes.

Only referenced assets are copied by default. Each copied file receives a content-hashed public name under `public/mambo/`, while original logical paths remain in source metadata for diagnostics. Identical content may be deduplicated.

Paths must remain within the configured content root after canonicalization. `..` traversal that escapes the root, device files, and symlinks escaping the root are errors. The first release must not search an external attachment directory to satisfy a missing asset; any synchronization workflow must materialize required assets inside `docs/` before compilation.

## Callouts

Obsidian callouts use blockquote syntax:

```md
> [!NOTE] Optional title
> Callout content with **Markdown**.
```

Supported initial kinds are `note`, `abstract`, `info`, `todo`, `tip`, `success`, `question`, `warning`, `failure`, `danger`, `bug`, `example`, and `quote`. Unknown kinds remain callouts with a neutral style and produce a warning, allowing Obsidian-authored content to remain readable.

Fold markers may be parsed for compatibility, but static output should render content expanded unless a runtime disclosure component is explicitly enabled.

## Comments and authoring-only constructs

Obsidian comments delimited by `%%` are removed from visible output and search text. Unclosed comments are errors because they can hide the remainder of a page unexpectedly.

Inline tags are preserved as text in the first release. Only frontmatter `tags` participate in taxonomy. Obsidian property widgets, Bases embeds, and plugin code blocks must not execute; an unsupported embed or executable block produces a clear diagnostic.

## Raw HTML and sanitization

Raw HTML is disabled by default. It should be escaped or rejected according to configuration, never passed through accidentally. Even in a future trusted mode, script, iframe, event-handler attributes, unsafe URLs, and style injection require explicit sanitization rules.

Generated TypeScript stores structured nodes rather than untrusted HTML. React rendering must avoid `dangerouslySetInnerHTML` for authored content.

## Resolution order and error policy

The compiler must complete global indexing before resolving references. Errors are accumulated where safe so one build reports multiple independent problems. Output is written only when there are no errors.

Resolution must never depend on:

- Filesystem enumeration order.
- Host path separator.
- Current locale.
- Random identifiers.
- Network access.
- JavaScript execution.

These guarantees are necessary for reproducible local and CI builds.

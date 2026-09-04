---
description: Rust parsing pipeline and rules for links, embeds, headings, and assets.
title: Parsing and Resolution
order: 40
---

# Parsing and Resolution

## Input guarantees

Every source file must be valid UTF-8. An optional UTF-8 byte-order mark is accepted before frontmatter. LF, CRLF, and CR line endings are accepted, and source diagnostics retain their line, column, and byte positions.

The compiler reads source without modifying it. Parsing and build operations must never rewrite source Markdown or mutate the configured content root.

## Content-root path model

MamboSite receives a repository-local content root, normally `docs/`, and a content-root-relative site entry, normally `index.md`, from `mambo.toml`. These configured paths define the complete namespace used by parsing and resolution.

Every source receives a logical path relative to the content root, normalized to forward slashes. Logical paths never contain an absolute prefix, `.` segments, escaping `..` segments, host-specific separators, or authoring-tool locations. Absolute host paths may appear only in internal I/O state; user diagnostics, generated TypeScript, and stable page identifiers use logical paths.

Ordinary discovery skips `_mounts/`, `_info.md`, `README.md`, and the other exclusions defined in [[Content Model]]. Each mount declared in the configured entry creates a second, explicit traversal beginning at a repository-local path such as `_mounts/mambodot/index.md`. This makes the selected subtree reachable without turning `_mounts` into a route segment or exposing unrelated stored projects.

Every mount source must remain within the content root after lexical normalization and filesystem canonicalization. Local note links and embeds resolve only against discovered pages. The compiler rejects absolute or root-escaping source paths and rejects symlinks anywhere in the content tree.

The reserved `_assets/` directory is not page content. A separate asset pass admits only regular contained files and resolves explicit `assets/<path>` references against that root.

MamboSite does not discover an Obsidian vault or synchronize authoring files. Project Mambo's optional materialization workflow is described in [[Documentation Sync]].

## Parsing stages

### 1. Entry and source discovery

The compiler loads `mambo.toml`, reads the entry frontmatter to obtain mounts, and performs deterministic ordinary and mount-specific directory walks. Logical paths are content-root relative and symlinks are rejected.

### 2. Frontmatter split

Frontmatter is recognized only when the first non-BOM line is exactly `---`. The closing delimiter must also be exactly `---` on its own line. A missing closing delimiter is an error.

YAML is deserialized into a generic JSON-compatible value and then validated into MamboSite fields. YAML anchors, aliases, merge keys, and explicit tags are rejected.

### 3. Markdown AST

Comrak parses the body with selected CommonMark/GFM and extension options. The adapter immediately lowers Comrak's arena-backed nodes into MamboSite's owned AST; no Comrak lifetime escapes it.

The owned AST retains node kind, child order, source span, heading levels, code-block information, and parsed directive data. Note destinations remain authored; explicit `assets/...` destinations are rewritten to validated public URLs during semantic resolution.

### 4. Directive and Obsidian lowering

Leaf directives are recognized only in eligible Markdown text nodes, so code blocks and inline code remain untouched. Container directives use Comrak block-directive nodes. MamboSite parses both forms with source spans and balanced strings/arrays.

The directive parser recognizes balanced quotes, arrays, and braces. A malformed directive is a syntax error even if CommonMark could otherwise treat it as text.

The dialect pass recognizes `![[...]]`, `%%` comments, and block IDs left in text by the base parser. Comrak supplies wikilink and alert nodes. These features do not give the compiler knowledge of a vault, `.obsidian/`, or an Obsidian installation.

### 5. Validation and semantic resolution

Resolution occurs only after every relevant file is indexed. This allows forward links, aliases, backlinks, route collision checks, and cycle detection to work across the entire site.

The compiler validates directives, headings, blocks, routes, and mount namespaces; derives titles, descriptions, page IDs, and direct children; resolves links, fragments, note embeds, backlinks, cycles, and depth limits; then validates and rewrites content-asset destinations. Generated navigation and search text are not derived yet.

## Internal node model

The current Rust intermediate representation is a renderer-neutral, serializable parser tree. Representative nodes include:

```text
Document
Paragraph
Heading
Text
Emphasis
Strong
Strikethrough
Highlight
InlineCode
CodeBlock
Link
Image
List
ListItem
BlockQuote
Alert
Table
ThematicBreak
FootnoteDefinition
FootnoteReference
Math
Directive
ObsidianEmbed
```

For note references, the parser AST remains authored syntax. Resolution stores normalized `outgoingLinks` and `embeds` beside that tree, keyed by destination and source span. Explicit `assets/...` references are the exception: they are rewritten in the AST, frontmatter, and validated directives to their compiled public URL and do not create note-graph edges. The runtime uses these compiler-resolved values instead of reparsing Markdown.

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

Relative `.md` links resolve through the content graph and are rewritten to site routes. Root-relative links are interpreted relative to the site root. Absolute HTTP(S), `mailto`, and `tel` links remain external; protocol-relative URLs are also retained.

Unsafe schemes such as `javascript:` and malformed control-character URLs are errors. External links are not fetched or validated during a normal build.

## Heading identifiers

Heading IDs are generated in Rust so the table of contents, links, embeds, and rendered HTML agree without client-side repair.

Rules:

1. Use normalized visible heading text.
2. Remove formatting while retaining its text.
3. Normalize Unicode to NFC, lowercase ASCII, and convert punctuation/whitespace runs to `-`.
4. Preserve non-ASCII letters and numbers.
5. If empty, use `section`.
6. Add `-2`, `-3`, and so on for duplicates in document order.

Explicit heading attributes and parent-heading records are not implemented. The generated heading index contains ID, text, level, and source span, and the runtime uses those IDs directly.

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

An embed is never implemented by copying or indenting Markdown text. The compiler retains an embed AST node and records its resolved page/fragment target as a graph edge.

### Default embed mode

The default runtime handles a plain `![[Page]]` as follows:

- Render as a visibly bounded embedded article/section determined by the theme.
- Keep source provenance on all nested nodes.
- Prefix generated DOM IDs with an embed-instance identifier to prevent collisions.
- Do not merge the embedded headings into the host page's main table of contents by default.

This is a semantic component boundary. The current theme may indent or border it, but physical Markdown indentation is never introduced.

### Inline mode

The default runtime supports whole-page `include` directives in `embed` or `inline` mode. `headings="shift"` adds one rendered heading level, `strip-title` omits top-level H1 nodes, and `keep` retains authored levels. Embedded IDs are prefixed and same-page fragment links are rewritten to that prefix.

Include sources are currently resolved by the runtime content store rather than stored as compiler-authoritative directive edges. Included headings are not added to the host page's generated heading index, and heading overflow is capped at H6 rather than diagnosed. Fragment includes are reported as unsupported.

### Cycles and limits

The compiler builds a directed graph for Obsidian note embeds. `A -> B -> A` and longer cycles are errors showing the route chain; `markdown.max_embed_depth`, default 16, limits acyclic chains. Runtime `include` recursion has a separate render-time guard because include targets are not compiler graph edges yet.

## Images and local assets

Supported examples from any page directory:

```md
![Alternative text](assets/mambo.png)
![[assets/mambo.png]]
![[assets/mambo.png|640]]
![[assets/mambo.png|640x360]]
```

The matching repository file is `_assets/mambo.png`. The fixed author namespace avoids page-relative guessing: `assets/mambo.png` always means that same file, including from nested articles. Standard Markdown images and links, wikilinks and Obsidian embeds, frontmatter `cover`, and `hero.image` or `button.href` values use this mapping when their destination starts with `assets/`.

For `assets_out = "public/mambo"`, compilation rewrites the example to `/mambo/assets/mambo.png`; the Next adapter adds `site.base_path` once when rendering. Missing files, escaping paths, symlinks, unsupported filesystem entries, and normalized path collisions are errors. A successful build copies the complete `_assets/` tree into the managed public output, so removed inputs do not survive the next build.

Schema 1 preserves alt text and Obsidian size options but does not yet interpret those size options, inspect media, hash filenames, transform files, or deduplicate identical bytes. Destinations outside the explicit `assets/` namespace remain ordinary links or site-owned public paths.

## Callouts

Obsidian callouts use blockquote syntax:

```md
> [!NOTE] Optional title
> Callout content with **Markdown**.
```

The current Comrak-backed schema supports `note`, `tip`, `important`, `warning`, and `caution`. Broader Obsidian kinds and fold markers are planned.

## Comments and authoring-only constructs

Obsidian comments delimited by `%%` are removed from the AST, so they are absent from visible output. Unclosed comments are errors because they can hide the remainder of a page unexpectedly.

Inline tags are preserved as text. Only frontmatter `tags` participate in current related-content queries. Obsidian property widgets, Bases embeds, and plugin code blocks are not executed; unsupported fenced blocks remain ordinary code.

## Raw HTML and sanitization

Raw HTML is preserved in the AST. With `markdown.raw_html = false`, the compiler warns, and the default renderer displays the source as code text rather than injecting it. Setting the flag currently suppresses that warning; trusted HTML rendering and sanitization are not implemented.

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

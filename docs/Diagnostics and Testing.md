---
title: Diagnostics and Testing
description: Error model, validation rules, fixtures, and acceptance gates for MamboSite.
order: 70
---

# Diagnostics and Testing

## Diagnostic goals

MamboSite should make content failures as understandable as compiler errors. Every actionable diagnostic should answer:

- What failed?
- Where did it fail?
- Why is it invalid or ambiguous?
- What related source contributed to the failure?
- What can the author change?

Diagnostics are structured values first and terminal text second. This permits human output, JSON output, CI annotations, and future editor integration from the same data.

## Diagnostic record

```text
severity: error | warning | note
code: stable identifier such as MS4102
message: concise primary message
primary span: logical path, line, column, byte range
secondary spans: related declarations or candidates
help: optional concrete correction
notes: optional resolution trace
```

Paths shown to users are normalized paths relative to the configured content root. Files from configured asset directories use a stable configured-root label plus a relative path. Absolute host paths are never shown. ANSI colour is enabled only for an interactive terminal or explicit request.

## Code families

| Range | Area |
|---|---|
| `MS1xxx` | Configuration and schema versions |
| `MS2xxx` | Discovery, paths, encoding, and frontmatter |
| `MS3xxx` | Markdown dialects, including Obsidian-compatible syntax, and directives |
| `MS4xxx` | Routes, hierarchy, navigation, and mounts |
| `MS5xxx` | Links, embeds, fragments, and assets |
| `MS6xxx` | TypeScript generation and output writing |
| `MS7xxx` | Runtime/build compatibility |

Codes remain stable after publication even if wording improves.

## Required validation

### Configuration

- Unsupported config schema.
- Missing or invalid content root, entry, or output path.
- Output directory overlapping source content.
- Absolute or escaping paths where forbidden.
- Invalid site URL or base path.
- Compiler/runtime schema incompatibility.

### Source and frontmatter

- Invalid UTF-8.
- Unclosed frontmatter.
- Invalid YAML or unsupported YAML values.
- Wrong core-field type.
- Invalid status, date, slug, alias, order, or mount.
- Unknown strict-mode top-level key.
- Duplicate alias that makes resolution ambiguous.
- Draft content required by a published embed.

### Markdown and directives

- Unclosed code fence, comment, or container directive where detectable.
- Malformed directive attributes.
- Unknown directive or property.
- Missing required property.
- Invalid property value.
- Duplicate `page` directive.
- Context errors such as `children` on a non-index page.
- Invalid columns/column nesting.
- Heading level or explicit-ID errors.
- Duplicate block ID.

### Routes and mounts

- Duplicate normalized route.
- Both `page.md` and `page/index.md`.
- Invalid route segment.
- Mount source not resolving to `index.md`.
- Duplicate, overlapping, or cyclic mounts.
- Physical/mounted route collision.
- Page with no valid route when publication requires one.

### Links and embeds

- Missing or ambiguous note target.
- Missing heading or block fragment.
- Internal link to an unrouted target.
- Unsafe URL scheme.
- Embed cycle or maximum-depth violation.
- Broken heading shift.
- Published page embedding draft/private content.

### Assets

- Missing or ambiguous asset.
- Asset path escaping the content root or a configured asset directory.
- Unsupported special file type.
- Symlink encountered under strict policy.
- Invalid image dimensions or malformed media.
- Generated public-name collision.

### Generation

- Invalid serialized value.
- Runtime schema mismatch.
- Failed temporary write or atomic replacement.
- Generated output escaping its configured directory.
- Stale-output cleanup attempted without a valid marker.

## Warning policy

Warnings identify valid but suspicious content:

- Missing explicit title or description.
- Multiple H1 headings.
- Heading hierarchy jump.
- Case-insensitive link fallback.
- Unknown callout kind.
- Unlisted page with no incoming links.
- Unused asset in a configured asset directory.
- Deprecated field or directive.
- Missing meaningful alt text for a content image.
- Raw HTML escaped or ignored.

Warnings do not change output. CI may use `--deny-warnings` or a configured list of warning codes for stricter repositories.

## Inspect mode

`mambosite inspect` should expose resolution decisions without enabling debug logs globally.

Example report areas:

```text
target
logical source path
mount and final route
title/description provenance
frontmatter normalization
parent and child pages
outgoing links and their resolution stage
backlinks
embed expansion tree
assets and public URLs
generated module path
```

This command is important for resolving Obsidian-compatible basename links and mount-context questions. That syntax is a supported input dialect, not a required authoring workflow.

## Test layers

### Unit tests

Pure tests for:

- Route and slug normalization.
- Frontmatter field validation.
- Directive attribute parsing.
- Heading and block IDs.
- Link target parsing.
- Path containment.
- Asset-name hashing.
- Deterministic sorting.
- TypeScript string escaping.

### Parser fixtures

Each syntax feature has small input Markdown plus an expected normalized AST or diagnostics file. Fixtures include Unicode, CRLF, nested formatting, code fences containing fake directives, malformed syntax, and Obsidian variants.

### Graph fixtures

Miniature repository fixtures, each with a configured content root, test:

- Physical hierarchy.
- Mounted hierarchy.
- Wiki links and aliases.
- Route collisions.
- Backlinks.
- Embed chains and cycles.
- Assets in configured directories outside the referring note's directory.
- Draft and unlisted behaviour.

### Golden TypeScript tests

Validated fixture projects generate complete expected TypeScript trees. Golden output catches accidental schema, ordering, escaping, and formatting changes. Updates require deliberate review rather than automatic acceptance.

### Runtime contract tests

The TypeScript package type-checks representative and generated fixtures. Every node variant and directive component must render or fail with an explicit schema mismatch.

### End-to-end tests

Small Folio- and Wiki-shaped example sites run the complete flow:

```text
repository fixture + configured content root -> Rust compile -> TypeScript typecheck -> Next static build -> inspect out/
```

Assertions cover generated routes, canonical URLs, internal links, asset existence, 404 output, base paths, and absence of server-only requirements.

### Browser and accessibility tests

The default runtime should test keyboard navigation, focus visibility, headings, link purpose, image alternatives, theme contrast, responsive collections, and reduced motion. Visual snapshots may protect the initial MamboFolio-inspired theme without making its exact appearance a compiler contract.

### Fuzz and property tests

Once core parsing works, fuzz directive/frontmatter/link parsing and assert that arbitrary UTF-8 never panics or writes outside configured output. Property tests should cover normalization idempotence and deterministic output.

## Fixture layout

```text
tests/fixtures/
├── valid/
│   ├── basic-page/
│   ├── wiki-mounts/
│   ├── embeds/
│   ├── assets/
│   └── directives/
└── invalid/
    ├── route-collision/
    ├── ambiguous-link/
    ├── embed-cycle/
    ├── malformed-directive/
    └── escaped-path/
```

Every invalid fixture declares expected diagnostic codes and primary paths. Tests should avoid matching entire prose messages unless wording itself is being tested.

## Quality gates

Before the first release:

- Rust formatting, linting, and tests pass.
- TypeScript linting, type checking, and tests pass.
- Golden outputs are clean.
- Both example sites produce static exports.
- Repeating a clean build produces byte-identical generated semantic output.
- Broken fixtures fail with their expected diagnostic codes and never write output.
- No compiler test requires network access.
- Path and symlink security cases are covered on supported platforms.
- The default theme meets the agreed accessibility baseline.

## Regression policy

Every fixed parser, resolver, route, embed, or generation bug receives the smallest practical fixture reproducing it. A bug in mounted content should be tested both at its content-root-relative source path and final site route when relevant.

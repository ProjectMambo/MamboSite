---
title: Diagnostics and Testing
description: Current error model, validation coverage, tests, and remaining quality gates.
order: 70
---

# Diagnostics and Testing

## Diagnostic model

Compiler diagnostics are structured values with these fields:

```text
severity: error | warning | note
code: stable identifier such as MS5102
message: concise primary message
primary: optional logical path and source span
related: zero or more related paths and spans
help: optional correction
notes: optional resolution context
```

Source spans contain one-based line/column positions and optional zero-based UTF-8 byte offsets. `mbsite --diagnostics json check` serializes compiler diagnostics as JSON; text is the default. Lifecycle, theme, subprocess, and output-publication failures currently use plain error messages rather than this structured schema.

Current diagnostic families are:

| Range | Current area |
|---|---|
| `MS1xxx` | Configuration and safe project paths |
| `MS2xxx` | Source discovery, UTF-8, and frontmatter |
| `MS3xxx` | Markdown, Obsidian syntax, and directives |
| `MS4xxx` | Routes and mounts |
| `MS5xxx` | Note links, fragments, and note embeds |
| `MST1xxx` | Theme schema and token validation |

Code-generation and lifecycle errors do not yet have `MS6xxx`/`MS7xxx` structured codes.

## Current validation coverage

MamboSite currently rejects:

- Unsupported or malformed configuration; unsafe/overlapping project paths; invalid site URL/base path; invalid renderer/deploy names.
- Missing content roots or entries, non-UTF-8 Markdown, and any content-tree symlink.
- Unclosed/invalid frontmatter, unsupported YAML constructs, wrong core-field types, malformed mounts, and unknown fields when frontmatter strict mode is enabled.
- Unclosed code fences or Obsidian comments/embeds, invalid or duplicate block IDs, and malformed/unknown/misnested directives.
- Empty or duplicate routes, invalid mount sources/routes, overlapping mount paths, and physical pages inside mounted namespaces.
- Missing or ambiguous note targets/fragments, unsafe URL schemes, and note-embed cycles or excessive depth.
- Invalid theme schemas/tokens and unsafe or unowned generated-output directories.

Current warnings are multiple H1 headings, heading-level jumps, raw HTML when disabled, and unresolved/ambiguous note references when `markdown.strict_links = false`. There is no `--deny-warnings` option yet.

Not yet validated:

- Date syntax or normalized taxonomy identifiers.
- Content-asset existence, media type, dimensions, containment, or public output.
- Draft-to-published reference policy.
- Compiler-authoritative targets for directive properties such as `include.source` and `button.href`.
- Accessibility, external links, or generated search/navigation records.

## Output safety

Compilation or theme-validation errors leave existing generated trees untouched. Each generated tree carries `.mambosite-generated`; replacement refuses unmarked non-empty directories, writes a temporary sibling, and restores the previous tree when publication fails. TypeScript and theme assets are published as two separately managed trees, not one cross-directory transaction.

## Current tests

Rust tests live beside their modules, with theme integration tests under `crates/mambosite-theme/tests/`. They cover configuration/path safety, frontmatter, route and mount discovery, Markdown lowering, directives, reference graphs, deterministic TypeScript generation, managed writers, theme compilation, CLI parsing, init safety, build orchestration, and deploy decisions.

The npm workspace has focused Node tests for:

- Generated schema compatibility and immutable content-store queries.
- Typed registry composition and rendering.
- Default-theme rendering and unsupported-mode behavior.
- Next.js route, metadata, base-path, image, and theme-bootstrap adapters.

Run the current gates with:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm run check:packages
npm run test:packages
git diff --check
```

## Remaining release tests

Before version 0.1 is complete, add only the fixtures needed to cover behavior that unit tests cannot prove:

- Checked-in valid/invalid repository fixtures and reviewed generated-output goldens.
- Clean end-to-end MamboFolio and MamboWiki static builds, including routes, base paths, 404 output, and public files.
- Browser checks for keyboard use, focus, heading structure, responsive layouts, contrast, and reduced motion.
- Property/fuzz coverage for parser inputs and path/output containment.
- A byte-for-byte repeated-build check.

`mbsite inspect` and CI warning escalation remain planned. Every parser, resolver, route, embed, or writer regression should receive the smallest test that reproduces it.

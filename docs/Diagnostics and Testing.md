---
description: Current error model, validation coverage, tests, and remaining quality gates.
title: Diagnostics and testing
order: 70
---

# Diagnostics and testing

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
| `MS5xxx` | Note links, fragments, note embeds, and content assets |
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
- Missing, escaping, symlinked, unsupported, or normalized-colliding content assets.
- Invalid theme schemas/tokens—including explicit empty or one-sided accent keys, mismatched custom palettes, and unsafe or unowned generated-output directories.

Current warnings are multiple H1 headings, heading-level jumps, raw HTML when disabled, and unresolved/ambiguous note references when `markdown.strict_links = false`. There is no `--deny-warnings` option yet.

Not yet validated:

- Date syntax or normalized taxonomy identifiers.
- Content-asset media type, intrinsic dimensions, content hashing, or transformation.
- Draft-to-published reference policy.
- Compiler-authoritative targets for directive properties such as `include.source` and non-asset `button.href` values.
- Accessibility, external links, or generated search/navigation records.

## Output safety

Compilation or theme-validation errors leave existing generated trees untouched. Each generated tree carries `.mambosite-generated`; replacement refuses unmarked non-empty directories, writes a temporary sibling, and restores the previous tree when publication fails. TypeScript and the combined theme/content-asset tree are published as two separately managed trees, not one cross-directory transaction.

## Current tests

Rust tests live beside their modules, with theme integration tests under `crates/mambosite-theme/tests/`. They cover configuration/path safety, frontmatter, route and mount discovery, Markdown lowering, directives, footer context, reference and asset resolution, binary asset publication, deterministic TypeScript generation, managed writers, build timestamps, MamboColour role mapping, complete paired provider-palette enumeration and seeded permutation, explicit custom-accent provenance and shuffle behavior, serialized/default scaffold omission of empty accent keys, rejection of explicit empty or one-sided accent inputs, absence of private palette/file coupling, expanded status-role contrast against both background and surface, CLI parsing, init safety, build orchestration, and deploy decisions. These tests retain regression coverage for provider-versus-custom provenance so key presence cannot silently change ownership.

The npm workspace has focused Node tests for:

- Generated schema compatibility and immutable content-store queries.
- Typed registry composition and rendering.
- Default-theme rendering, card top-accent borders, static timezone timestamps, footer content, and unsupported-mode behavior.
- Next.js route, metadata, base-path, image, and theme-bootstrap adapters.

Run the current gates with:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm run check:packages
npm run test:packages
./script/test_install.sh
../MamboDocs/script/check-repository.sh --strict .
git diff --check
```

Maintainers changing bundled MamboFont assets also run `npm run sync:theme:check`. That command invokes only the pinned MamboFont provider and fails when the checked-in web fonts or stylesheet are stale. MamboColour is an ordinary pinned Cargo dependency; its compatibility boundary is exercised by the Rust theme and workspace tests rather than an asset-sync command.

## Remaining release tests

Before version 0.1 is complete, add only the fixtures needed to cover behavior that unit tests cannot prove:

- Checked-in valid/invalid repository fixtures and reviewed generated-output goldens.
- Clean end-to-end MamboFolio and MamboWiki static builds, including routes, base paths, 404 output, and public files.
- Browser checks for keyboard use, focus, heading structure, responsive layouts, contrast, and reduced motion.
- Property/fuzz coverage for parser inputs and path/output containment.
- A byte-for-byte repeated-build check with `SOURCE_DATE_EPOCH` fixed.

`mbsite inspect` and CI warning escalation remain planned. Every parser, resolver, route, embed, or writer regression should receive the smallest test that reproduces it.

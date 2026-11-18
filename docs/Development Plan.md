---
description: Current evidence, dependency-ordered stages, exit gates, risks, and delivery milestones for MamboSite.
title: Development plan
order: 15
---

::page{layout="docs" width="normal" sidebar=true}

# Development plan

MamboSite is an active Markdown-first static-site platform. Its Rust compiler, generated TypeScript contract, React runtime, default theme, Next.js adapter, and repository lifecycle commands already build and deploy MamboFolio and MamboWiki. This plan starts from that released foundation and orders the remaining work by semantic dependency and delivery risk.

The plan is authoritative for intended delivery order, not a promise of calendar dates. Update it when implementation evidence changes a public contract, trust boundary, cross-repository dependency, stage boundary, or release gate.

## Current state

Evidence reviewed for this plan includes the `v0.1.2` source release, current `main`, the compiler and package test inventories, synchronized documentation, and the deployed MamboFolio and MamboWiki consumers.

| Area | Current evidence | Remaining gap |
|---|---|---|
| Content and configuration | Schema 1 discovers repository-local Markdown, validates frontmatter and paths, derives routes, and composes entry-owned mounts | Date syntax, normalized taxonomy identifiers, and draft-to-published reference policy are not enforced |
| Markdown and graph | CommonMark/GFM, supported Obsidian syntax, links, embeds, blocks, backlinks, routes, mounts, and assets compile with source-aware diagnostics | Directive targets still rely partly on runtime lookup; fragment includes and some heading edge cases remain incomplete |
| Generated output | Rust emits typed page and manifest modules plus managed theme and asset trees; fixed `SOURCE_DATE_EPOCH` makes timestamps and accent selection reproducible | Search/navigation records, compiler-version build metadata, sitemaps, and RSS are not emitted |
| Runtime and presentation | Versioned runtime, React registry, default theme, and Next.js adapter render the modes used by MamboFolio and MamboWiki | Tree/table child views, nested child depth, masonry/carousel galleries, and fragment includes still show explicit unsupported states |
| Lifecycle | `mbsite check`, `build`, `init`, and guarded GitHub Pages `deploy` are implemented; output replacement and scaffold refresh protect unowned files | `inspect`, warning escalation, migration checks, and a low-friction watch workflow remain absent |
| Distribution | Source releases `v0.1.0` through `v0.1.2` exist and consumers pin exact MamboSite commits | The four `@mambosite/*` packages remain workspace-local, so consumers still use sibling `file:` dependencies |
| Quality | Rust and package tests, installer checks, and strict MamboDocs validation form the complete repository gate; clean consumer builds provide contract-change and release evidence | Checked-in whole-site goldens, browser accessibility coverage, parser/path fuzzing, and an automated repeated-build comparison remain release work |

The current source release is `v0.1.2`. Compatible improvements on `main` add static build timestamps, footer-authored directives, smoother shared motion, complete provider colour cycling, and brand-coloured authored emphasis without changing generated schema 1.

## Target outcome

The target is a supported schema-1 platform where a site owner can:

1. create or configure a site from a clean checkout;
2. author or mount Markdown and receive compiler-owned diagnostics for every internal target;
3. render every default-theme mode accepted by schema 1 without a hidden runtime approximation;
4. generate discoverable static output with correct base-path and draft/unlisted handling;
5. install compatible released web packages instead of sibling `file:` dependencies;
6. validate, build, deploy, and roll back MamboFolio or MamboWiki from exact provider versions.

The complete acceptance journey starts from a clean consumer clone, compiles representative authored and mounted content, checks failure fixtures, exports the static site, verifies keyboard and responsive behavior, deploys the exact commit, and restores the previous provider pin when rollback is exercised.

## Scope and deferrals

This plan includes schema-1 reference correctness, accepted default-renderer modes, search and static-discovery records, actionable inspection, release-grade fixtures and browser checks, npm package publication, and provider-before-consumer rollout.

The following remain outside the target unless evidence changes the boundary:

- MDX, arbitrary authored components, remote content fetching, a CMS, authentication, comments, forms, or a runtime server;
- trusted raw-HTML rendering without an explicit sanitization and threat model;
- an Obsidian-specific authoring dependency or ownership of cross-repository vault synchronization;
- third-party Rust parser plugins, a general theme-plugin marketplace, or a second framework adapter without a real consumer;
- image transformation, asset fingerprinting, or specialized audio/video/PDF rendering without a demonstrated site requirement;
- incremental compilation, a file watcher, or editor protocol until measured authoring latency justifies their lifecycle complexity;
- automatic external-link crawling or browser-time content fetching;
- a new generated-schema version until an accepted feature cannot remain backward-compatible with schema 1.

## Architecture and ownership

~~~text
repository Markdown + mambo.toml + mambo.theme.toml
    -> mambosite-core parses, resolves, and validates
    -> mambosite-theme compiles semantic theme data and CSS
    -> mambosite-codegen-ts publishes managed generated trees
    -> mambosite-cli coordinates check/build/init/deploy
    -> @mambosite/runtime indexes immutable compiled content
    -> @mambosite/react renders typed nodes and directives
    -> @mambosite/theme-default supplies layouts and components
    -> @mambosite/next adapts links, images, metadata, and routes
    -> consumer-owned static export and GitHub Pages deployment
~~~

MamboSite owns compiler semantics, schema compatibility, generated output, shared presentation, and lifecycle safety. A consumer owns its Markdown, site configuration, optional typed registry overrides, public URL, workflow, and deployment approval. MamboColour owns palette data and its Rust API; MamboFont owns font generation; MamboSite pins those provider contracts and commits the reviewed assets or dependency revision it consumes. The Project Mambo notes workflow may materialize canonical documentation, but MamboSite consumes only the resulting repository-local tree.

Authored Markdown, YAML/TOML, paths, URLs, mounted trees, and existing output directories are untrusted inputs. The compiler must preserve path containment, reject symlinks and unsafe schemes, keep raw HTML inert by default, and mutate only marked generated destinations. Static output is public by design; neither diagnostics nor build logs may reveal credentials or content beyond the paths and excerpts required to fix an error.

Cross-repository changes move provider first: MamboColour or MamboFont when their public contracts change, then MamboSite, then MamboFolio and MamboWiki pins and deployments. Documentation is authored canonically, synchronized to MamboSite, and finally published through the MamboWiki mount.

## Stage 0: Released end-to-end foundation — complete

The released foundation established schema-1 compilation, typed generation, the package compatibility boundary, provider-backed theme tokens, safe lifecycle commands, and clean static deployments for both current consumers.

Prerequisites: the canonical content model, directive grammar, generated schema, and static-hosting boundary were frozen before implementation.

Reach: Rust crates, four TypeScript packages, default scaffold, installer, MamboFolio and MamboWiki integrations, source-release documentation, and regression tests.

Risk controls: source spans and stable diagnostics protect authors; managed-output markers and atomic replacement protect files; exact provider and consumer pins protect compatibility; raw HTML is not injected.

Focused checks: the complete MamboSite repository gate, clean MamboFolio and MamboWiki production builds, GitHub Pages workflow results, and inspection of the exact source tag and consumer pins.

Exit gate — met: source release `v0.1.2` exists, both consumers exported and deployed the pinned schema-1 platform, and the documented repository gates passed. Post-release theme work remains schema-1 compatible.

Deliberately left out: incomplete directive-edge semantics, optional renderer modes, discovery output, authoring inspection, and public npm distribution move through the stages below instead of being relabelled as shipped.

## Stage 1: Make schema 1 compiler-authoritative

Outcome: every internal target and policy accepted by schema 1 is resolved or rejected by Rust before generation; the runtime consumes decisions instead of guessing author intent.

Prerequisites: decide whether the additional resolved-edge fields are backward-compatible optional schema-1 data. If an existing consumer cannot safely ignore them, introduce an explicit generated-schema upgrade and migration rather than silently changing schema 1.

Reach:

- resolve `children.source`, `children.include`, `gallery.source`, `include.source`, and internal button destinations against the mounted route graph;
- record typed directive edges and provenance in generated output;
- validate date syntax, normalized taxonomy identifiers, draft-to-published references, include heading overflow, and unsafe or ambiguous directive targets;
- add `--deny-warnings` while preserving current text and JSON diagnostic behavior;
- update core/codegen/runtime types, fixtures, diagnostics, authoring references, and compatibility documentation together.

Risks: a generated-type change can strand consumers; mounted and source-relative targets can resolve differently; warning escalation can break existing CI. Compatibility tests and consumer fixtures must cover all three boundaries.

Focused checks: core reference/directive tests, TypeScript generation goldens, runtime schema tests, JSON diagnostic snapshots, and MamboFolio/MamboWiki content checks.

Exit gate: broken, ambiguous, escaping, draft-forbidden, or unsupported directive targets fail with stable source-aware diagnostics; valid physical and mounted references generate typed targets; `--deny-warnings` is deterministic; neither default runtime nor consumer overrides need to parse note-style references.

Left out: fragment semantics remain Stage 2, search and navigation records remain Stage 3, and optional presentation modes remain Stage 4.

## Stage 2: Complete fragment composition

Outcome: fragment includes and embeds use compiler-authoritative targets and render predictable static content instead of an unsupported placeholder.

Prerequisites: Stage 1 must provide typed directive edges and its generated-schema compatibility decision. Fragment composition must reuse the compiler's IDs, provenance, and cycle protection.

Reach:

- select heading and block fragments in Rust and retain their source provenance;
- render fragment includes and embeds with namespaced IDs, bounded heading shifts, ordinary fragment links, and cycle safety;
- define whether included headings enter the host heading index and table of contents, then encode that decision in generated fixtures;
- preserve a readable source link when a fragment cannot be rendered;
- update core/codegen/runtime models, default components, static-render tests, fixtures, and authoring documentation together.

Risks: duplicate DOM IDs, recursive content, heading overflow, and fragment links that point at the wrong embedded instance. Composition must remain static and deterministic.

Focused checks: compiler fragment and cycle fixtures, TypeScript output goldens, React static markup, repeated-embed anchor tests, and clean MamboFolio/MamboWiki builds.

Exit gate: valid heading and block fragments render the selected content with unique stable IDs; missing, ambiguous, recursive, or overflowing fragments fail before publication or produce the one documented safe fallback; repeated embeds preserve ordinary anchor behavior in both consumers.

Left out: collection and gallery presentation remain Stage 4; code highlighting, transformed media, and arbitrary layout plugins remain deferred.

## Stage 3: Generate discovery and static-publication data

Outcome: Rust owns the deterministic records needed for navigation, search, canonical metadata, sitemaps, and feeds; static adapters serialize those records without rediscovering content semantics.

Prerequisites: Stages 1 and 2 must settle final routes, visibility, fragments, and page relationships. Search record contents and draft/unlisted policy must be documented before generation. Collection and gallery presentation does not block this data contract.

Reach:

- emit normalized navigation and search records with stable ordering;
- exclude drafts, hidden footer content, comments, and configured non-searchable material;
- add compiler version and schema information to build metadata;
- generate canonical URL, sitemap, and RSS inputs with correct `site.url`, `base_path`, and trailing-slash behavior;
- add an accessible default search surface only after the record contract is proven by fixtures;
- update output types, Next metadata integration, templates, tests, and operator documentation.

Risks: private or draft text can leak into public indexes; base paths can be applied twice; feed/search ordering can become nondeterministic. Generated artifacts must be inspectable and contain only explicitly public content.

Focused checks: output goldens for custom-domain and project-path sites, draft/unlisted fixtures, schema compatibility, static route assertions, and search keyboard behavior.

Exit gate: both consumers publish correct canonical URLs and deterministic discovery artifacts; draft/unlisted policies are proven by negative fixtures; search finds representative mounted and local pages without browser-time Markdown parsing.

Left out: remote indexing, external-link crawling, analytics, personalized ranking, and a server-backed search service.

## Stage 4: Complete accepted collection and gallery modes

Outcome: the default theme renders every schema-1 collection and gallery mode it accepts, with accessible semantics and no unsupported placeholder for valid core content.

Prerequisites: Stage 1 must provide compiler-authoritative child and gallery targets; Stage 2 must settle composition semantics used by nested content. Discovery records from Stage 3 remain presentation-independent.

Reach:

- implement tree and table child views plus documented nested child depth;
- implement masonry and keyboard-operable carousel galleries or narrow the public contract through an explicit versioned migration;
- preserve useful static fallback content when client behavior is unavailable;
- update React models, default components, styles, static-render tests, fixtures, and authoring/theme documentation.

Risks: hydration drift, motion sensitivity, focus loss, and inaccessible tree, table, masonry, or carousel semantics. Prefer semantic static HTML and the smallest client behavior that preserves keyboard use.

Focused checks: React static markup, component behavior tests, valid and empty-state fixtures, keyboard/focus checks, reduced-motion checks, and clean MamboFolio/MamboWiki builds.

Exit gate: every documented schema-1 collection and gallery view has a valid and empty-state fixture, works at narrow and wide layouts, preserves readable static output, and passes keyboard, focus, and reduced-motion review in both consumers.

Left out: code highlighting, transformed media, arbitrary layout plugins, and cosmetic redesigns not required by these contracts.

## Stage 5: Add actionable authoring inspection

Outcome: a contributor can explain compiled behavior and migrate a consumer without reading generated modules or compiler internals.

Prerequisites: Stages 1 through 4 must establish the authoritative edge, composition, discovery, and presentation contracts that inspection reports.

Reach:

- implement `mbsite inspect <target>` for source path, route, wikilink, metadata derivation, mount origin, children, links, embeds, assets, directives, and diagnostics;
- keep text output human-readable and provide stable JSON for tooling;
- refresh scaffold examples and add migration checks for released configuration, generated schema, and package pins;
- preserve the existing `mbsite init --force` ownership invariant while adding migration checks and new scaffold content;
- document the smallest author-check-preview-deploy and upgrade journeys.

Risks: inspection can accidentally expose absolute paths or become a second compiler. It must report existing compiler state, redact irrelevant host details, and perform no writes.

Focused checks: CLI help and invalid-target tests, text/JSON snapshots, init refresh fixtures, released-version migration fixtures, and clean-clone scaffold builds.

Exit gate: a maintainer can explain a local or mounted page and one failed reference, upgrade fixtures from the current and immediately previous published source releases, and refresh a scaffold without changing an unknown file.

Left out: watch mode, a language server, editor-specific integrations, and incremental compilation until measured workflow latency justifies them.

## Stage 6: Prove release-grade quality and recovery

Outcome: repository and consumer gates demonstrate correctness, containment, accessibility, reproducibility, and rollback before a distribution change.

Prerequisites: Stages 1 through 5 must be complete enough that fixtures describe the intended supported surface rather than temporary behavior.

Reach:

- check in representative valid and invalid repository fixtures plus reviewed generated-output goldens;
- run clean end-to-end MamboFolio and MamboWiki builds covering routes, mounts, assets, base paths, 404 output, discovery files, and public files;
- compare two fixed-epoch builds byte for byte;
- add bounded property or fuzz coverage for parser inputs, path containment, directive attributes, and generated-output paths;
- run browser checks for keyboard navigation, focus, heading structure, contrast, responsive layouts, theme switching, and reduced motion;
- exercise installer protection, failed generated-tree publication, init recovery, deploy dry-run, and provider-pin rollback.

Risks: golden fixtures can become noisy, fuzzing can create unbounded CI time, and screenshots can hide semantic failures. Keep fixtures small, seeds and limits reproducible, and accessibility assertions semantic.

Focused checks: the complete repository gate below, consumer `npm run check`, deterministic-build comparison, bounded fuzz/property jobs, and browser smoke tests against production artifacts.

Exit gate: all gates pass from clean clones; injected parser, renderer, output-publication, and deployment failures leave authored content and previous managed output intact; the previous exact provider pin restores both consumers.

Left out: arbitrary performance promises. Add a performance budget only after representative site measurements identify a real regression boundary.

## Stage 7: Publish packages and roll out consumers

Outcome: compatible `@mambosite/runtime`, `@mambosite/react`, `@mambosite/theme-default`, and `@mambosite/next` packages are published together, and consumers no longer require sibling `file:` dependencies for the web runtime.

Prerequisites: Stage 6 must pass; package exports, peer ranges, licences, provenance, release notes, and registry ownership must be reviewed. Publication and release require explicit maintainer approval.

Reach:

- align the Rust workspace, npm packages, scaffold pins, source tag, and compatibility notes;
- publish the four packages as one reviewed compatibility unit;
- retain one documented exact-source compiler installation until verified binary distribution has its own integrity and rollback design;
- update MamboFolio and MamboWiki manifests, lockfiles, workflows, canonical docs, and provider pins in provider-before-consumer order;
- deploy consumers sequentially and verify the public routes before declaring rollout complete.

Risks: partial package publication, peer-version mismatch, registry credential exposure, source/package drift, and consumer rollback failure. A failed partial publish must not be labelled as a supported compatibility set.

Focused checks: packed-package inspection, clean installs using only published web packages, full MamboSite gate, both consumer gates, exact tag/package comparison, and live Pages smoke checks.

Exit gate: a fresh consumer clone installs the released web packages, builds with the documented exact compiler source, deploys successfully, and can revert to the previous compatible package and compiler pins without content migration.

Left out: `mbsite release`, automatic publication from an unreviewed commit, and binary artifacts without signing, integrity, platform, and recovery decisions.

## Data, compatibility, and recovery

MamboSite owns no database and must never modify authored Markdown during check or build. `init --force` may replace only scaffold-manifest-owned paths. Generated trees are disposable caches with ownership markers; a failed validation or publication keeps the previous tree, and a clean rebuild is the recovery path.

Configuration schema, frontmatter interpretation, directive grammar, generated data, and runtime support are versioned contracts. Backward-compatible optional fields may remain within schema 1 only when older consumers can ignore them safely. Breaking syntax or required generated fields need an explicit schema increase, migration documentation, compatibility fixtures, and a consumer rollout plan.

Migration fixtures cover the current and immediately previous published source releases. At this planning baseline those releases are `v0.1.2` and `v0.1.1`; older tags remain historical evidence rather than supported upgrade baselines. Change that window only with explicit lifecycle documentation and retained recovery guidance.

The four npm packages move together as a compatibility unit even though they remain independently named. Source tags and npm publication are distinct events and must identify the same reviewed contract. Consumers recover by restoring their previous compiler/package pins and lockfile, rebuilding static output, and deploying a normal revert; published history is never rewritten.

## Security and privacy

- Reject escaping paths, symlinks, normalized collisions, unsafe URL schemes, malformed directives, and unowned output directories before mutation.
- Keep raw HTML inert unless a future explicit sanitization contract replaces the current policy.
- Treat Markdown, embedded text, filenames, metadata, and diagnostic excerpts as data, never shell input.
- Invoke renderers and Git as argument arrays without a shell and keep deploy restricted to a clean configured branch and workflow.
- Do not read network content during compilation or initialization.
- Keep GitHub and registry credentials outside generated files and logs; use provider-native short-lived credentials where publication supports them.
- Warn authors that every compiled page, asset, search record, feed item, and source excerpt placed in the static output becomes public.

## Validation strategy

Use the smallest affected crate or package test while editing. The complete MamboSite gate is:

~~~bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm run check:packages
npm run test:packages
./script/test_install.sh
../MamboDocs/script/check-repository.sh --strict .
git diff --check
~~~

Provider updates also run the provider-specific reproducibility or compatibility checks documented in the README. Stages that change generated contracts run clean MamboFolio and MamboWiki checks at exact pins. Stages 2 through 4 and Stage 6 add production-artifact browser review. Release candidates add clean installs, packed-package inspection, fixed-epoch repeated builds, deploy dry-runs, and the complete acceptance and rollback journeys.

## Delivery milestones

| Milestone | Included stages | Observable outcome |
|---|---|---|
| Released baseline | Stage 0 | Source `v0.1.2` and live schema-1 consumers |
| Compiler-owned schema 1 | Stage 1 | Every accepted internal target and policy is resolved before generation |
| Resolved composition | Stage 2 | Fragment includes and embeds have deterministic compiler-owned behavior |
| Discoverable static platform | Stage 3 | Deterministic public discovery output with correct visibility and URLs |
| Complete default presentation | Stage 4 | Every accepted collection and gallery mode has accessible static behavior |
| Actionable authoring platform | Stage 5 | Inspection and supported-release migration from a clean scaffold |
| Release candidate | Stage 6 | Clean-clone, reproducibility, recovery, accessibility, and consumer evidence |
| Distributed platform | Stage 7 | Published compatible packages and verified consumer rollout/rollback |

No milestone advances while an earlier exit gate remains open. A source release or package publication is a separate approved action after the applicable milestone passes; completing documentation does not authorize one.

## Known risks and evidence triggers

| Risk or assumption | Current treatment | Trigger that changes the plan |
|---|---|---|
| Directive-edge data may not fit schema 1 safely | Decide compatibility before Stage 1 implementation | An older supported runtime cannot ignore or reproduce the new field |
| Runtime feature breadth could outrun real sites | Finish already accepted contracts; defer new modes | A second consumer demonstrates a missing semantic mode |
| Search may add unnecessary client weight | Generate small records first and measure | Current consumers exceed an agreed payload or interaction budget |
| Build latency may hurt authoring | Keep deterministic full builds | Representative measured edits make the supported loop impractical |
| Asset processing can expand scope and supply-chain risk | Copy validated local assets unchanged | A supported site needs dimensions, transforms, or fingerprinted caching |
| A second framework adapter can split compatibility work | Keep Next.js as the only adapter | A maintained consumer commits to another static framework and its tests |
| Package publication can drift from source | Release one reviewed compatibility unit | Registry constraints require independent cadence or version ranges |
| Raw HTML can expose readers to script injection | Render source as code text | A documented trusted-content use case includes a reviewed sanitizer boundary |

The next implementation stage is Stage 1. Work should not begin with search UI, package publication, watch mode, or visual additions while compiler-authoritative directive targets and schema compatibility remain unsettled.

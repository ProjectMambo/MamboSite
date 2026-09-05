import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { renderToStaticMarkup } from "react-dom/server";
import { createElement } from "react";

import { createMamboRuntime, MamboPage } from "@mambosite/react";
import { defaultRegistry } from "../dist/index.js";

test("default styles do not duplicate generated theme values as literal fallbacks", async () => {
  const css = await readFile(new URL("../src/styles/default.css", import.meta.url), "utf8");
  const callsWithFallbacks = cssVariableCalls(css)
    .filter(({ fallback }) => fallback !== null)
    .map(({ name, fallback }) => [name, fallback]);

  assert.deepEqual(callsWithFallbacks, [["--mambo-card-fit", "cover"]]);
  assert.match(
    css,
    /\.mambo-site-header__inner\s*\{[^}]*display: var\(--mambo-header-display\)/s,
  );
  assert.doesNotMatch(css, /\.mambo-site-header\s*\{[^}]*display:/s);
  assert.doesNotMatch(css, /\.mambo-site-navigation\s*\{[^}]*overflow-x: auto/s);
  assert.match(
    css,
    /\.mambo-site-navigation\s*\{[^}]*display: var\(--mambo-header-navigation-display\)[^}]*flex-direction: var\(--mambo-header-navigation-direction\)[^}]*position: var\(--mambo-header-navigation-position\)/s,
  );
  assert.match(
    css,
    /\.mambo-site-navigation\[data-open="true"\]\s*\{[^}]*max-block-size: calc\(100dvh - var\(--mambo-dimension-header-height\)\)[^}]*overflow-y: auto/s,
  );
  assert.match(
    css,
    /\.mambo-navigation-toggle\s*\{[^}]*display: var\(--mambo-header-toggle-display\)/s,
  );
  assert.match(css, /\.mambo-theme-toggle > span\s*\{[^}]*font-size: 1\.5em/s);
  assert.match(css, /\.mambo-tooltip:has\(:focus-visible\) \.mambo-tooltip__content/);
  assert.match(css, /@media \(hover: hover\)[\s\S]*\.mambo-tooltip:hover/);
  assert.doesNotMatch(css, /\.mambo-tooltip:focus-within/);
  assert.match(css, /a:active\s*\{[^}]*--mambo-color-brand-active/s);
  assert.match(css, /\.mambo-site-header:has\([^}]*\{[^}]*translateY\(0\)/s);
  assert.match(css, /\.mambo-back-link--top\s*\{[^}]*grid-column: 1 \/ -1/s);
  assert.match(css, /\.mambo-page-sidebar--inline \.mambo-toc__disclosure\s*\{[^}]*display: block/s);
  assert.match(css, /\.mambo-page-sidebar--rail \.mambo-toc__expanded\s*\{[^}]*display: block/s);
  assert.match(css, /\.mambo-toc a\[aria-current="location"\]/);
  assert.match(css, /\.mambo-collection\[data-view="cards"\]/);
  assert.match(css, /\[data-layout="gallery"\][^{]*\{[^}]*--mambo-width-gallery-image-max/s);
  assert.doesNotMatch(css, /minmax\(min\(100%, var\(--mambo-width-card-min\)/);
});

test("header renders an accessible compact menu and custom theme tooltip", () => {
  const page = compiledPage({
    id: "p_header",
    route: "/",
    data: {
      navigation: [
        { label: "Brand", href: "/" },
        { label: "Docs", href: "/docs/" },
      ],
    },
  });
  const manifest = {
    schemaVersion: 1,
    site: {
      title: "Fixture",
      basePath: "",
      language: "en-SG",
      trailingSlash: true,
    },
    entryPage: page.id,
    routes: { "/": page.id },
    pages: [page],
  };
  const runtime = createMamboRuntime({ manifest, pages: [page], registry: defaultRegistry });
  const Header = defaultRegistry.shell.Header;
  const html = renderToStaticMarkup(createElement(Header, { runtime }));
  const navigationId = html.match(/aria-controls="([^"]+)"/)?.[1];
  const tooltipId = html.match(/aria-describedby="([^"]+)"/)?.[1];

  assert.ok(navigationId);
  assert.ok(tooltipId);
  assert.match(html, /aria-expanded="false"/);
  assert.match(html, new RegExp(`id="${navigationId}"`));
  assert.match(html, /data-open="false"/);
  assert.match(html, new RegExp(`id="${tooltipId}" role="tooltip"`));
  assert.match(html, />Change colour theme<\/span>/);
  assert.doesNotMatch(html, /title="Change colour theme"/);
});

test("default theme renders compiled Markdown and resolved directives", () => {
  const directiveSpan = {
    start: { line: 2, column: 1 },
    end: { line: 2, column: 10 },
    startByte: 10,
    endByte: 19,
  };
  const page = {
    schemaVersion: 1,
    id: "p_root",
    route: "/",
    sourcePath: "index.md",
    title: "Mambo fixture",
    description: "A renderer fixture.",
    status: "published",
    listed: true,
    tags: ["web/test"],
    aliases: [],
    data: {},
    extra: {},
    headings: [],
    blocks: [],
    directives: [{
      name: "hero",
      form: "leaf",
      properties: {
        align: { type: "string", value: "center" },
        "show-title": { type: "boolean", value: true },
        "show-description": { type: "boolean", value: true },
        "show-meta": { type: "boolean", value: false },
      },
      span: directiveSpan,
    }],
    body: {
      type: "document",
      children: [{
        type: "directive",
        invocation: {
          name: "hero",
          form: "leaf",
          properties: [],
          span: { start: 10, end: 19 },
          nameSpan: { start: 12, end: 16 },
          raw: "::hero{}",
        },
        span: directiveSpan,
      }, {
        type: "paragraph",
        children: [{ type: "text", value: "Compiled body" }],
      }],
    },
    children: [],
    outgoingLinks: [],
    embeds: [],
    backlinks: [],
  };
  const manifest = {
    schemaVersion: 1,
    site: {
      title: "Fixture",
      basePath: "",
      language: "en-SG",
      trailingSlash: true,
    },
    entryPage: page.id,
    routes: { "/": page.id },
    pages: [page],
  };
  const runtime = createMamboRuntime({ manifest, pages: [page], registry: defaultRegistry });
  const html = renderToStaticMarkup(createElement(MamboPage, { runtime, page }));

  assert.match(html, /mambo-hero--center/);
  assert.match(html, /Mambo fixture/);
  assert.match(html, /Compiled body/);
});

test("collection markup exposes requested columns without overriding responsive theme CSS", async () => {
  const { CollectionView } = await import("../dist/index.js");
  const page = {
    id: "p_card",
    route: "/card/",
    title: "Card",
    description: "Card description",
    status: "published",
    listed: true,
    tags: [],
    aliases: [],
    data: {},
    children: [],
  };
  const runtime = {
    registry: defaultRegistry,
    options: {},
  };
  const html = renderToStaticMarkup(createElement(CollectionView, {
    items: [page],
    runtime,
    view: "grid",
    columns: 9,
  }));
  assert.match(html, /data-columns="6"/);
  assert.match(html, /data-mambo-accent-item="true"/);
  assert.doesNotMatch(html, /--mambo-collection-columns/);

  const list = renderToStaticMarkup(createElement(CollectionView, {
    items: [page],
    runtime,
    view: "list",
    columns: 6,
  }));
  assert.match(list, /data-columns="1"/);

  const cards = renderToStaticMarkup(createElement(CollectionView, {
    items: [page],
    runtime,
    view: "cards",
    show: ["title"],
  }));
  assert.match(cards, /data-view="cards"/);
  assert.doesNotMatch(cards, /Card description/);
});

test("a hero hides the generated title only when its validated show-title is true", () => {
  const directiveSpan = sourceSpan(2, 10, 19);
  const page = compiledPage({
    id: "p_hero_without_title",
    route: "/hero-without-title/",
    title: "Generated title remains",
    directives: [{
      name: "hero",
      form: "leaf",
      properties: {
        "show-title": { type: "boolean", value: false },
        "show-description": { type: "boolean", value: false },
        "show-meta": { type: "boolean", value: false },
        align: { type: "string", value: "left" },
      },
      span: directiveSpan,
    }],
    body: {
      type: "document",
      children: [{
        type: "directive",
        invocation: {
          name: "hero",
          form: "leaf",
          properties: [],
          span: { start: 10, end: 19 },
          nameSpan: { start: 12, end: 16 },
          raw: "::hero{}",
        },
        span: directiveSpan,
      }],
    },
  });

  const html = renderCompiledPage(page);
  assert.match(html, /<h1>Generated title remains<\/h1>/);
  assert.equal(html.match(/<h1>/g)?.length, 1);
});

test("layouts render useful TOCs and predictable back controls", () => {
  for (const layout of ["article", "gallery"]) {
    const withoutHeadings = pageWithLayout(layout);
    const withoutToc = renderCompiledPage(withoutHeadings);
    assert.doesNotMatch(withoutToc, /class="mambo-toc"/);
    assert.doesNotMatch(withoutToc, /class="mambo-page-sidebar"/);

    const withHeadings = pageWithLayout(layout, { heading: true });
    const withToc = renderCompiledPage(withHeadings);
    assert.match(withToc, /class="mambo-toc"/);
    assert.equal(withToc.match(/class="mambo-page-sidebar /g)?.length, 2);
    assert.match(
      withToc,
      new RegExp(`mambo-page-frame--${layout === "article" ? "normal" : "wide"}`),
    );
    assert.ok(withToc.indexOf('data-mambo-back="top"') < withToc.indexOf("mambo-page-sidebar"));
    assert.ok(withToc.indexOf("mambo-page-sidebar") < withToc.indexOf("mambo-page-article"));
    assert.ok(withToc.indexOf("mambo-page-article") < withToc.lastIndexOf("mambo-page-sidebar"));
    assert.equal(withToc.match(/class="mambo-toc__expanded"/g)?.length, 2);
    assert.equal(withToc.match(/class="mambo-toc__disclosure"/g)?.length, 2);
  }

  const nested = pageWithLayout("project", { route: "/project/example/" });
  const backLinks = renderCompiledPage(nested);
  assert.equal(backLinks.match(/data-mambo-back=/g)?.length, 2);
  assert.equal(backLinks.match(/href="\/project\/"/g)?.length, 2);

  const authoredToc = pageWithLayout("article", { heading: true, authoredToc: true });
  const singleToc = renderCompiledPage(authoredToc);
  assert.equal(singleToc.match(/class="mambo-toc"/g)?.length, 1);
  assert.doesNotMatch(singleToc, /class="mambo-page-sidebar"/);
  assert.match(singleToc, /data-collapse="false"/);

  const docsToc = renderCompiledPage(pageWithLayout("docs", { heading: true, sidebar: false }));
  assert.match(docsToc, /class="mambo-toc"/);
});

test("Back delegates to native history so the browser owns scroll restoration", async () => {
  const source = await readFile(
    new URL("../src/layouts/PageBackBehavior.tsx", import.meta.url),
    "utf8",
  );
  assert.match(source, /window\.history\.back\(\)/);
  assert.doesNotMatch(source, /sessionStorage|scrollTo\(/);
});

test("two embeds namespace same-page fragment links with their generated DOM ids", () => {
  const firstEmbedSpan = sourceSpan(2, 10, 20);
  const secondEmbedSpan = sourceSpan(3, 21, 31);
  const headingSpan = sourceSpan(1, 0, 9);
  const linkSpan = sourceSpan(2, 10, 28);
  const target = compiledPage({
    id: "p_embedded_target",
    route: "/target/",
    sourcePath: "target.md",
    title: "Target",
    headings: [{ id: "section", level: 1, text: "Section", span: headingSpan }],
    outgoingLinks: [{
      syntax: "markdown",
      authoredDestination: "#section",
      target: {
        kind: "page",
        pageId: "p_embedded_target",
        route: "/target/",
        fragment: { kind: "heading", id: "section" },
      },
      span: linkSpan,
    }],
    body: {
      type: "document",
      children: [{
        type: "heading",
        level: 1,
        setext: false,
        span: headingSpan,
        children: [{ type: "text", value: "Section" }],
      }, {
        type: "paragraph",
        children: [{
          type: "link",
          destination: "#section",
          title: "",
          span: linkSpan,
          children: [{ type: "text", value: "Jump locally" }],
        }],
      }],
    },
  });
  const host = compiledPage({
    id: "p_embed_host",
    route: "/",
    sourcePath: "index.md",
    title: "Host",
    embeds: [{
      authoredDestination: "target",
      instanceId: "e_one",
      target: { kind: "page", pageId: target.id, route: target.route },
      span: firstEmbedSpan,
    }, {
      authoredDestination: "target",
      instanceId: "e_two",
      target: { kind: "page", pageId: target.id, route: target.route },
      span: secondEmbedSpan,
    }],
    body: {
      type: "document",
      children: [{
        type: "obsidianEmbed",
        destination: "target",
        option: null,
        span: firstEmbedSpan,
      }, {
        type: "obsidianEmbed",
        destination: "target",
        option: null,
        span: secondEmbedSpan,
      }],
    },
  });

  const html = renderCompiledPage(host, [host, target]);
  assert.match(html, /id="e_one-section"/);
  assert.match(html, /href="#e_one-section"/);
  assert.match(html, /id="e_two-section"/);
  assert.match(html, /href="#e_two-section"/);
});

function sourceSpan(line, startByte, endByte) {
  return {
    start: { line, column: 1 },
    end: { line, column: endByte - startByte + 1 },
    startByte,
    endByte,
  };
}

function compiledPage(overrides = {}) {
  return {
    schemaVersion: 1,
    id: "p_fixture",
    route: "/fixture/",
    sourcePath: "fixture.md",
    title: "Fixture",
    status: "published",
    listed: true,
    tags: [],
    aliases: [],
    data: {},
    extra: {},
    headings: [],
    blocks: [],
    directives: [],
    body: { type: "document" },
    children: [],
    outgoingLinks: [],
    embeds: [],
    backlinks: [],
    ...overrides,
  };
}

function pageWithLayout(
  layout,
  { heading = false, authoredToc = false, route = `/${layout}/example/`, sidebar } = {},
) {
  const pageSpan = sourceSpan(1, 0, 20);
  const tocSpan = sourceSpan(2, 21, 29);
  const headingSpan = sourceSpan(3, 30, 42);
  const directives = [{
    name: "page",
    form: "leaf",
    properties: {
      layout: { type: "string", value: layout },
      ...(sidebar === undefined ? {} : { sidebar: { type: "boolean", value: sidebar } }),
    },
    span: pageSpan,
  }];
  const children = [{
    type: "directive",
    invocation: {
      name: "page",
      form: "leaf",
      properties: [],
      span: { start: 0, end: 20 },
      nameSpan: { start: 2, end: 6 },
      raw: `::page{layout="${layout}"}`,
    },
    span: pageSpan,
  }];
  if (authoredToc) {
    directives.push({ name: "toc", form: "leaf", properties: {}, span: tocSpan });
    children.push({
      type: "directive",
      invocation: {
        name: "toc",
        form: "leaf",
        properties: [],
        span: { start: 21, end: 29 },
        nameSpan: { start: 23, end: 26 },
        raw: "::toc{}",
      },
      span: tocSpan,
    });
  }
  if (heading) {
    children.push({
      type: "heading",
      level: 2,
      setext: false,
      span: headingSpan,
      children: [{ type: "text", value: "Details" }],
    });
  }
  return compiledPage({
    id: `p_${layout}_${heading ? "heading" : "plain"}_${authoredToc ? "authored" : "auto"}`,
    route,
    title: `${layout} fixture`,
    directives,
    headings: heading
      ? [{ id: "details", level: 2, text: "Details", span: headingSpan }]
      : [],
    body: { type: "document", children },
  });
}

function renderCompiledPage(page, pages = [page]) {
  const manifest = {
    schemaVersion: 1,
    site: {
      title: "Fixture",
      basePath: "",
      language: "en-SG",
      trailingSlash: true,
    },
    entryPage: page.id,
    routes: Object.fromEntries(pages.map((item) => [item.route, item.id])),
    pages,
  };
  const runtime = createMamboRuntime({ manifest, pages, registry: defaultRegistry });
  return renderToStaticMarkup(createElement(MamboPage, { runtime, page }));
}

function cssVariableCalls(css) {
  const calls = [];
  const startPattern = /var\(\s*(--mambo-[\w-]+)/g;
  let match;

  while ((match = startPattern.exec(css)) !== null) {
    let depth = 1;
    let comma = -1;
    let cursor = startPattern.lastIndex;

    for (; cursor < css.length && depth > 0; cursor += 1) {
      if (css[cursor] === "(") depth += 1;
      if (css[cursor] === ")") depth -= 1;
      if (css[cursor] === "," && depth === 1 && comma === -1) comma = cursor;
    }

    calls.push({
      name: match[1],
      fallback: comma === -1 ? null : css.slice(comma + 1, cursor - 1).trim(),
    });
  }

  return calls;
}

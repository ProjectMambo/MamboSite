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

  assert.deepEqual(callsWithFallbacks, [
    ["--mambo-color-accent-2", "var(--mambo-color-accent-1)"],
    ["--mambo-color-accent-3", "var(--mambo-color-accent-1)"],
    ["--mambo-color-accent-4", "var(--mambo-color-accent-1)"],
    ["--mambo-color-accent-5", "var(--mambo-color-accent-1)"],
    ["--mambo-color-accent-6", "var(--mambo-color-accent-1)"],
    ["--mambo-card-fit", "cover"],
  ]);
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
  assert.doesNotMatch(html, /--mambo-collection-columns/);
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

import assert from "node:assert/strict";
import test from "node:test";
import { renderToStaticMarkup } from "react-dom/server";
import { createElement } from "react";

import { createMamboRuntime, MamboPage } from "@mambosite/react";
import { defaultRegistry } from "../dist/index.js";

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

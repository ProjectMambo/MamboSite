import assert from "node:assert/strict";
import test from "node:test";

import {
  createContentStore,
  resolvedDirectiveProperties,
} from "../dist/index.js";

const span = {
  start: { line: 2, column: 1 },
  end: { line: 2, column: 18 },
  startByte: 10,
  endByte: 27,
};

const child = page({
  id: "p_child",
  route: "/child/",
  sourcePath: "child.md",
  title: "Child",
});
const root = page({
  id: "p_root",
  route: "/",
  sourcePath: "index.md",
  title: "Root",
  children: [child.id],
  outgoingLinks: [{
    syntax: "wiki",
    authoredDestination: "child",
    target: {
      kind: "page",
      pageId: child.id,
      route: child.route,
      fragment: { kind: "heading", id: "intro" },
    },
    span,
  }],
  embeds: [{
    authoredDestination: "child",
    instanceId: "e_fixture",
    target: { kind: "page", pageId: child.id, route: child.route },
    span,
  }],
  directives: [{
    name: "hero",
    form: "leaf",
    properties: {
      align: { type: "string", value: "center" },
      "show-title": { type: "boolean", value: true },
    },
    span,
  }],
});
const manifest = {
  schemaVersion: 1,
  site: {
    title: "Fixture",
    basePath: "",
    language: "en-SG",
    trailingSlash: true,
  },
  entryPage: root.id,
  routes: { "/": root.id, "/child/": child.id },
  pages: [root, child],
};

test("content stores resolve routes, references and child pages", () => {
  const store = createContentStore({ manifest, pages: [root, child] });
  assert.equal(store.entryPage, root);
  assert.equal(store.getPageByRoute("/child"), child);
  assert.equal(store.resolvePageReference("child.md", root), child);
  assert.deepEqual(store.childPages(root), [child]);
  assert.equal(store.resolvedHref(root, "child", span), "/child/#intro");
  assert.equal(store.resolvedEmbed(root, "child", span)?.instanceId, "e_fixture");
  assert.ok(Object.isFrozen(store));
  assert.ok(Object.isFrozen(store.pages));
});

test("content stores reject incompatible generated schemas", () => {
  assert.throws(
    () => createContentStore({
      manifest: { ...manifest, schemaVersion: 99 },
      pages: [root, child],
    }),
    /schema 99/,
  );
});

test("schema-1 directives use compiler-validated defaults by source span", () => {
  const node = {
    type: "directive",
    invocation: {
      name: "hero",
      form: "leaf",
      properties: [{
        name: "align",
        value: { type: "string", value: "left" },
        span: { start: 0, end: 0 },
        nameSpan: { start: 0, end: 0 },
        valueSpan: { start: 0, end: 0 },
        raw: "align=left",
      }],
      span: { start: 10, end: 27 },
      nameSpan: { start: 12, end: 16 },
      raw: "::hero{}",
    },
    span,
  };
  assert.deepEqual(resolvedDirectiveProperties(root, node), {
    align: "center",
    "show-title": true,
  });
});

function page(overrides) {
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

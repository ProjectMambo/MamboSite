import assert from "node:assert/strict";
import test from "node:test";

import {
  createContentStore,
  deepFreeze,
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

test("content stores deeply freeze generated content", () => {
  const mutablePage = page({
    id: "p_immutable",
    route: "/immutable/",
    data: { nested: { enabled: true } },
    body: {
      type: "document",
      children: [{ type: "paragraph", children: [{ type: "text", value: "Original" }] }],
    },
  });
  const mutableManifest = {
    ...manifest,
    site: { ...manifest.site, title: "Immutable" },
    entryPage: mutablePage.id,
    routes: { "/immutable/": mutablePage.id },
    pages: [mutablePage],
  };
  const store = createContentStore({ manifest: mutableManifest, pages: [mutablePage] });

  assert.ok(Object.isFrozen(store.manifest.site));
  assert.ok(Object.isFrozen(store.pages[0].data.nested));
  assert.ok(Object.isFrozen(store.pages[0].body.children));
  assert.throws(() => {
    store.manifest.site.title = "Changed";
  }, TypeError);
  assert.throws(() => {
    store.pages[0].data.nested.enabled = false;
  }, TypeError);
  assert.throws(() => {
    store.pages[0].body.children[0].children[0].value = "Changed";
  }, TypeError);
});

test("deepFreeze handles cycles", () => {
  const cyclic = { nested: {} };
  cyclic.nested.parent = cyclic;
  assert.equal(deepFreeze(cyclic), cyclic);
  assert.ok(Object.isFrozen(cyclic));
  assert.ok(Object.isFrozen(cyclic.nested));
});

test("page references prefer normalized exact paths and reject escaping fallbacks", () => {
  const source = page({
    id: "p_source",
    route: "/guides/current/",
    sourcePath: "guides/current/index.md",
  });
  const relativeTarget = page({
    id: "p_relative",
    route: "/guides/target/",
    sourcePath: "guides/target.md",
  });
  const basenameCollision = page({
    id: "p_collision",
    route: "/other/target/",
    sourcePath: "other/target.md",
  });
  const aliased = page({
    id: "p_alias",
    route: "/named/",
    sourcePath: "named.md",
    aliases: ["Special Name"],
  });
  const referenceManifest = {
    ...manifest,
    entryPage: source.id,
    routes: Object.fromEntries(
      [source, relativeTarget, basenameCollision, aliased].map((item) => [item.route, item.id]),
    ),
    pages: [source, relativeTarget, basenameCollision, aliased],
  };
  const store = createContentStore({
    manifest: referenceManifest,
    pages: [source, relativeTarget, basenameCollision, aliased],
  });

  assert.equal(store.resolvePageReference("../target", source), relativeTarget);
  assert.equal(store.resolvePageReference("other/./target.md", source), basenameCollision);
  assert.equal(store.resolvePageReference("special name", source), aliased);
  assert.equal(store.resolvePageReference("../../../outside", source), undefined);
  assert.equal(store.resolvePageReference("target", undefined), undefined);
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

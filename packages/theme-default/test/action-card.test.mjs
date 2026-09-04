import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

import { defaultRegistry } from "../dist/index.js";

test("card buttons remain links", () => {
  const Button = defaultRegistry.directives.button;
  const runtime = { options: {}, registry: defaultRegistry };
  const html = renderToStaticMarkup(createElement(Button, {
    config: {
      external: true,
      href: "https://example.com/contact",
      label: "Contact",
      variant: "card",
    },
    model: { href: "https://example.com/contact" },
    runtime,
  }));

  assert.match(html, /class="mambo-button mambo-button--card"/);
  assert.match(html, /href="https:\/\/example\.com\/contact"/);
  assert.match(html, /target="_blank"/);
});

test("card buttons inherit the generated collection accent", async () => {
  const css = await readFile(new URL("../src/styles/default.css", import.meta.url), "utf8");

  assert.doesNotMatch(css, /nth-child\([^)]*\)[^{]*--mambo-card-accent/s);
  assert.match(
    css,
    /\.mambo-button--card\s*\{[^}]*border-block-start-color: var\(--mambo-card-accent\)/s,
  );
  assert.match(
    css,
    /\.mambo-button--card:hover\s*\{[^}]*border-color: var\(--mambo-card-accent\)/s,
  );

  const Columns = defaultRegistry.directives.columns;
  const Column = defaultRegistry.directives.column;
  const column = createElement(Column, { children: "Contact" });
  const html = renderToStaticMarkup(createElement(Columns, {
    children: column,
    config: { collapseAt: "md", count: 3, gap: "normal" },
  }));
  assert.match(html, /data-columns="3"/);
  assert.match(html, /data-mambo-accent-item="true"/);
});

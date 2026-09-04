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

test("card buttons reuse the collection accent cycle", async () => {
  const css = await readFile(new URL("../src/styles/default.css", import.meta.url), "utf8");

  assert.match(
    css,
    /\.mambo-content-card,\s*\.mambo-button--card\s*\{[^}]*--mambo-card-accent/s,
  );
  for (const position of ["6n + 2", "6n + 3", "6n + 4", "6n + 5", "6n"]) {
    assert.ok(css.includes(
      `.mambo-columns > .mambo-column:nth-child(${position}) > .mambo-button-row > .mambo-button--card`,
    ));
  }
  assert.match(
    css,
    /\.mambo-button--card\s*\{[^}]*border-block-start-color: var\(--mambo-card-accent\)/s,
  );
  assert.match(
    css,
    /\.mambo-button--card:hover\s*\{[^}]*border-color: var\(--mambo-card-accent\)/s,
  );
});

import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

import {
  activeHeadingIndex,
  headingIndexForScrollIntent,
} from "../dist/directives/TableOfContentsBehavior.js";

test("scrollspy follows heading geometry", () => {
  assert.equal(activeHeadingIndex([], 80), -1);
  assert.equal(activeHeadingIndex([200, 500, 700], 80), -1);
  assert.equal(activeHeadingIndex([-100, 60, 500], 80), 1);
  assert.equal(activeHeadingIndex([-400, 300, 700], 80), 0);
});

test("continued bottom scrolling advances progressively and upward intent resets", () => {
  assert.equal(headingIndexForScrollIntent(1, 1, 1, 5), 2);
  assert.equal(headingIndexForScrollIntent(1, 2, 1, 5), 3);
  assert.equal(headingIndexForScrollIntent(1, 4, 1, 5), 4);
  assert.equal(headingIndexForScrollIntent(3, 1, 1, 5), 3);
  assert.equal(headingIndexForScrollIntent(1, 3, -1, 5), 1);
});

test("clicked entries stay current until fragment scrolling settles", async () => {
  const source = await readFile(
    new URL("../src/directives/TableOfContentsBehavior.tsx", import.meta.url),
    "utf8",
  );

  assert.match(source, /clickedIndex = index;[\s\S]*show\(index\);/);
  assert.match(source, /show\(clickedIndex \?\? bottomIndex \?\? geometricIndex\(\)\)/);
  assert.match(
    source,
    /clickedIndex !== undefined && window\.location\.hash === clickedHash[\s\S]*else reset\(\)/,
  );
  assert.match(source, /addEventListener\("hashchange", handleHashChange\)/);
  assert.match(source, /addEventListener\("scrollend", settleClick\)/);
});

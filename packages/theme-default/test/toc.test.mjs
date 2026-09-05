import assert from "node:assert/strict";
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

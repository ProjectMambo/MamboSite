import assert from "node:assert/strict";
import test from "node:test";

import { activeHeadingIndex } from "../dist/directives/TableOfContentsBehavior.js";

test("scrollspy selects passed headings and the final short section at page bottom", () => {
  assert.equal(activeHeadingIndex([], 80, false), -1);
  assert.equal(activeHeadingIndex([200, 500, 700], 80, false), -1);
  assert.equal(activeHeadingIndex([-100, 60, 500], 80, false), 1);
  assert.equal(activeHeadingIndex([-400, 300, 700], 80, true), 2);
});

import assert from "node:assert/strict";
import test from "node:test";

import { prefixBasePath, themeBootstrapScript } from "../dist/index.js";

test("base paths apply exactly once to root-relative local images", () => {
  assert.equal(prefixBasePath("/mambo/image.png", "/project"), "/project/mambo/image.png");
  assert.equal(prefixBasePath("/project/mambo/image.png", "/project/"), "/project/mambo/image.png");
  assert.equal(prefixBasePath("https://example.com/image.png", "/project"), "https://example.com/image.png");
  assert.equal(prefixBasePath("data:image/png;base64,abc", "/project"), "data:image/png;base64,abc");
  assert.equal(prefixBasePath("//cdn.example.com/image.png", "/project"), "//cdn.example.com/image.png");
});

test("theme bootstrap uses and safely escapes the generated default scheme", () => {
  const script = themeBootstrapScript("dark</script>");
  assert.match(script, /dark\\u003c\/script>/);
  assert.doesNotMatch(script, /<\/script>/);
});

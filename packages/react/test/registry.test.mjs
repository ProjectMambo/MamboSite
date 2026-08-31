import assert from "node:assert/strict";
import test from "node:test";

import { createRegistry, defineRegistry } from "../dist/index.js";

const component = () => null;

test("registry overrides are isolated and deeply immutable at registry boundaries", () => {
  const base = defineRegistry({
    primitives: { Link: component, Image: component },
    nodes: { paragraph: component },
    directives: { hero: component },
    layouts: { default: component },
    shell: { Header: component, Footer: component, NotFound: component },
    fallbacks: { Node: component, Directive: component, Layout: component },
  });
  const replacement = () => "replacement";
  const derived = createRegistry(base, { directives: { hero: replacement } });

  assert.equal(base.directives.hero, component);
  assert.equal(derived.directives.hero, replacement);
  assert.notEqual(derived.directives, base.directives);
  assert.ok(Object.isFrozen(derived));
  assert.ok(Object.isFrozen(derived.directives));
  assert.throws(() => {
    derived.directives.hero = component;
  }, TypeError);
});

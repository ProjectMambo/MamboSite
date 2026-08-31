import assert from "node:assert/strict";
import test from "node:test";

import {
  createMamboRuntimeFromStore,
  createRegistry,
  defineRegistry,
} from "../dist/index.js";

const component = () => null;
const baseRegistry = defineRegistry({
  primitives: { Link: component, Image: component },
  nodes: { paragraph: component },
  directives: { hero: component },
  layouts: { default: component },
  shell: { Header: component, Footer: component, NotFound: component },
  fallbacks: { Node: component, Directive: component, Layout: component },
});

test("registry overrides are isolated and deeply immutable at registry boundaries", () => {
  const replacement = () => "replacement";
  const derived = createRegistry(baseRegistry, { directives: { hero: replacement } });

  assert.equal(baseRegistry.directives.hero, component);
  assert.equal(derived.directives.hero, replacement);
  assert.notEqual(derived.directives, baseRegistry.directives);
  assert.ok(Object.isFrozen(derived));
  assert.ok(Object.isFrozen(derived.directives));
  assert.throws(() => {
    derived.directives.hero = component;
  }, TypeError);
});

test("runtime options are deeply immutable between components", () => {
  const schemes = ["dark", "light"];
  const runtime = createMamboRuntimeFromStore({
    store: {},
    registry: baseRegistry,
    options: {
      locale: "en-SG",
      theme: { defaultScheme: "dark", schemes },
    },
  });

  assert.ok(Object.isFrozen(runtime.options));
  assert.ok(Object.isFrozen(runtime.options.theme));
  assert.ok(Object.isFrozen(runtime.options.theme.schemes));
  assert.throws(() => runtime.options.theme.schemes.push("contrast"), TypeError);
});

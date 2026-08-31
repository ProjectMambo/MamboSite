import type { MamboComponentRegistry, RegistryOverrides } from "./types.js";

/** Define and freeze a complete component registry. */
export function defineRegistry(registry: MamboComponentRegistry): MamboComponentRegistry {
  return freezeRegistry(registry);
}

/** Preserve keyed override inference without widening component prop types. */
export function defineOverrides(overrides: RegistryOverrides): RegistryOverrides {
  return overrides;
}

/**
 * Create a new immutable registry by replacing only explicitly supplied keys.
 * The base registry is never mutated, so multiple sites can safely share it.
 */
export function createRegistry(
  base: MamboComponentRegistry,
  ...overrides: readonly RegistryOverrides[]
): MamboComponentRegistry {
  const merged = overrides.reduce<MamboComponentRegistry>(
    (registry, override) => ({
      primitives: { ...registry.primitives, ...override.primitives },
      nodes: { ...registry.nodes, ...override.nodes },
      directives: { ...registry.directives, ...override.directives },
      layouts: { ...registry.layouts, ...override.layouts },
      shell: { ...registry.shell, ...override.shell },
      fallbacks: { ...registry.fallbacks, ...override.fallbacks },
    }),
    base,
  );
  return freezeRegistry(merged);
}

function freezeRegistry(registry: MamboComponentRegistry): MamboComponentRegistry {
  return Object.freeze({
    primitives: Object.freeze({ ...registry.primitives }),
    nodes: Object.freeze({ ...registry.nodes }),
    directives: Object.freeze({ ...registry.directives }),
    layouts: Object.freeze({ ...registry.layouts }),
    shell: Object.freeze({ ...registry.shell }),
    fallbacks: Object.freeze({ ...registry.fallbacks }),
  });
}

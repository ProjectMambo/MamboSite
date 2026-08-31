import {
  createContentStore,
  deepFreeze,
  type ContentStore,
  type PageRecord,
  type SiteManifest,
} from "@mambosite/runtime";
import type {
  MamboComponentRegistry,
  MamboRuntime,
  MamboRuntimeOptions,
} from "./types.js";

export interface CreateMamboRuntimeInput {
  readonly manifest: SiteManifest;
  readonly pages: readonly PageRecord[];
  readonly registry: MamboComponentRegistry;
  readonly options?: MamboRuntimeOptions;
}

export interface CreateMamboRuntimeFromStoreInput {
  readonly store: ContentStore;
  readonly registry: MamboComponentRegistry;
  readonly options?: MamboRuntimeOptions;
}

export function createMamboRuntime({
  manifest,
  pages,
  registry,
  options = {},
}: CreateMamboRuntimeInput): MamboRuntime {
  return createMamboRuntimeFromStore({
    store: createContentStore({ manifest, pages }),
    registry,
    options,
  });
}

export function createMamboRuntimeFromStore({
  store,
  registry,
  options = {},
}: CreateMamboRuntimeFromStoreInput): MamboRuntime {
  return Object.freeze({
    store,
    registry,
    options: deepFreeze({ ...options }),
  });
}

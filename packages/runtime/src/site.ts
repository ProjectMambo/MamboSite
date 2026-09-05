import type { PageId, PageRecord, PageSummary } from "./page.js";
import type { SiteMetadata } from "./metadata.js";

interface SiteBase {
  readonly schemaVersion: number;
  readonly generatedAt?: number;
  readonly site: SiteMetadata;
  readonly entryPage: PageId;
  readonly routes: Readonly<Record<string, PageId>>;
}

/** The compact site-wide module emitted as `manifest.ts`. */
export interface SiteManifest extends SiteBase {
  readonly pages: readonly PageSummary[];
}

/** The complete compiler model before page bodies are split into modules. */
export interface CompiledSite extends SiteBase {
  readonly pages: readonly PageRecord[];
}

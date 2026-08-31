import type { JsonObject } from "./json.js";

export type PageStatus = "published" | "draft";

export interface Mount {
  readonly path: string;
  readonly source: string;
}

/** Normalized author metadata before the compiler derives page fields. */
export interface PageMetadata {
  readonly title?: string;
  readonly description?: string;
  readonly slug?: string;
  readonly status: PageStatus;
  readonly listed: boolean;
  readonly date?: string;
  readonly updated?: string;
  readonly tags: readonly string[];
  readonly aliases: readonly string[];
  readonly order?: number;
  readonly cover?: string;
  readonly mounts: readonly Mount[];
  readonly data: JsonObject;
  /** Unknown compatibility values retained by the compiler. */
  readonly extra: JsonObject;
}

export interface SiteMetadata {
  readonly title: string;
  readonly url?: string;
  readonly basePath: string;
  readonly language: string;
  readonly trailingSlash: boolean;
}

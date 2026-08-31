import type { JsonObject } from "./json.js";
import type { ValidatedDirective } from "./directive.js";
import type { MarkdownNode } from "./markdown.js";
import type { PageStatus } from "./metadata.js";
import type { ResolvedEmbed, ResolvedLink } from "./reference.js";
import type { SourceSpan } from "./source.js";

export type PageId = string;

export interface HeadingRecord {
  readonly id: string;
  readonly level: number;
  readonly text: string;
  readonly span?: SourceSpan;
}

export interface BlockRecord {
  readonly id: string;
  readonly span?: SourceSpan;
}

/** A complete generated page module. */
export interface PageRecord {
  readonly schemaVersion: number;
  readonly id: PageId;
  readonly route: string;
  readonly sourcePath: string;
  readonly title: string;
  readonly description?: string;
  readonly status: PageStatus;
  readonly listed: boolean;
  readonly date?: string;
  readonly updated?: string;
  readonly tags: readonly string[];
  readonly aliases: readonly string[];
  readonly order?: number;
  readonly cover?: string;
  readonly data: JsonObject;
  readonly extra: JsonObject;
  readonly headings: readonly HeadingRecord[];
  readonly blocks: readonly BlockRecord[];
  readonly directives: readonly ValidatedDirective[];
  readonly body: MarkdownNode;
  readonly children: readonly PageId[];
  readonly outgoingLinks: readonly ResolvedLink[];
  readonly embeds: readonly ResolvedEmbed[];
  readonly backlinks: readonly PageId[];
}

/** The page subset embedded in `manifest.ts`. */
export interface PageSummary {
  readonly schemaVersion: number;
  readonly id: PageId;
  readonly route: string;
  readonly sourcePath: string;
  readonly title: string;
  readonly description?: string;
  readonly status: PageStatus;
  readonly listed: boolean;
  readonly date?: string;
  readonly updated?: string;
  readonly tags: readonly string[];
  readonly aliases: readonly string[];
  readonly order?: number;
  readonly cover?: string;
  readonly data: JsonObject;
  readonly children: readonly PageId[];
}

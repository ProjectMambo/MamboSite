import type { PageId } from "./page.js";
import type { SourceSpan } from "./source.js";

/** Authoring syntax retained for diagnostics and graph inspection. */
export type ReferenceSyntax = "markdown" | "wiki";

/** A fragment validated against the resolved target page. */
export type ResolvedFragment =
  | {
    readonly kind: "heading";
    readonly id: string;
  }
  | {
    readonly kind: "block";
    readonly id: string;
  };

/** The compiler result for one link destination. */
export type ResolvedLinkTarget =
  | {
    readonly kind: "page";
    readonly pageId: PageId;
    readonly route: string;
    readonly fragment?: ResolvedFragment;
  }
  | {
    readonly kind: "external";
    readonly href: string;
  }
  | {
    /** Present only when non-strict link validation retains a broken link. */
    readonly kind: "unresolved";
  };

/** A normalized outgoing-link graph edge. */
export interface ResolvedLink {
  readonly syntax: ReferenceSyntax;
  readonly authoredDestination: string;
  readonly target: ResolvedLinkTarget;
  readonly span?: SourceSpan;
}

/** A resolved Obsidian note embed edge before structural expansion. */
export interface ResolvedEmbed {
  readonly authoredDestination: string;
  readonly option?: string;
  readonly instanceId: string;
  readonly target: ResolvedLinkTarget;
  readonly span?: SourceSpan;
}

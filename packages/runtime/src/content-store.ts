import type { JsonObject, JsonValue } from "./json.js";
import type { PageRecord } from "./page.js";
import type { ResolvedEmbed, ResolvedLink } from "./reference.js";
import type { SourceSpan } from "./source.js";
import type { SiteManifest } from "./site.js";
import { assertCompatibleSchema } from "./compatibility.js";

export interface ContentStoreInput {
  readonly manifest: SiteManifest;
  readonly pages: readonly PageRecord[];
}

/**
 * An immutable, site-independent view of compiler output.
 *
 * Components receive this API instead of importing a generated manifest. That
 * keeps the renderer reusable and makes content queries straightforward to
 * test without a framework or filesystem.
 */
export interface ContentStore {
  readonly manifest: SiteManifest;
  readonly pages: readonly PageRecord[];
  readonly entryPage: PageRecord;
  getPageById(id: string): PageRecord;
  getPageByRoute(route: string): PageRecord | undefined;
  resolvePageReference(reference: string, sourcePage?: PageRecord): PageRecord | undefined;
  childPages(page: PageRecord, source?: string): readonly PageRecord[];
  sortPages(
    pages: readonly PageRecord[],
    key: string,
    direction?: "asc" | "desc" | string,
  ): readonly PageRecord[];
  resolvedLink(page: PageRecord, destination: string, span?: SourceSpan): ResolvedLink | undefined;
  resolvedEmbed(page: PageRecord, destination: string, span?: SourceSpan): ResolvedEmbed | undefined;
  resolvedHref(page: PageRecord, destination: string, span?: SourceSpan): string;
}

/** Recursively freezes compiler-shaped data without recursing forever on cycles. */
export function deepFreeze<T>(value: T): T {
  return freezeValue(value, new WeakSet<object>());
}

export function createContentStore({ manifest, pages }: ContentStoreInput): ContentStore {
  assertCompatibleSchema(manifest.schemaVersion, "the generated site manifest");
  for (const page of pages) {
    assertCompatibleSchema(page.schemaVersion, `generated page ${page.id}`);
  }
  const immutableManifest = deepFreeze(manifest);
  const immutablePages = deepFreeze([...pages]);
  const pagesById = new Map(immutablePages.map((page) => [page.id, page]));
  const pagesByRoute = new Map(
    immutablePages.map((page) => [
      normalizeRoute(page.route, immutableManifest.site.trailingSlash),
      page,
    ]),
  );
  const pagesBySource = indexPages(immutablePages, (page) =>
    normalizeSourceReference(page.sourcePath));
  const pagesByBasename = indexPages(immutablePages, (page) => {
    const source = normalizeSourceReference(page.sourcePath);
    return source?.slice(source.lastIndexOf("/") + 1);
  });
  const pagesByAlias = new Map<string, PageRecord[]>();
  for (const page of immutablePages) {
    for (const alias of page.aliases) addIndexedPage(pagesByAlias, normalizeAlias(alias), page);
  }

  const getPageById = (id: string): PageRecord => {
    const page = pagesById.get(id);
    if (!page) throw new Error(`Generated page ${id} is missing`);
    return page;
  };

  const getPageByRoute = (route: string): PageRecord | undefined =>
    pagesByRoute.get(normalizeRoute(route, immutableManifest.site.trailingSlash));

  const resolvePageReference = (
    reference: string,
    sourcePage?: PageRecord,
  ): PageRecord | undefined => {
    const authoredReference = reference.trim();
    let target = authoredReference;
    if (target.startsWith("[[") && target.endsWith("]]")) {
      target = target.slice(2, -2);
    }
    target = target.split("|", 1)[0]?.split("#", 1)[0]?.trim() ?? "";
    if (!target) return sourcePage;
    if (target.startsWith("/")) return getPageByRoute(target);

    const compiled = sourcePage?.outgoingLinks.find(
      (link) =>
        (link.authoredDestination === authoredReference || link.authoredDestination === target) &&
        link.target.kind === "page",
    );
    if (compiled?.target.kind === "page") return pagesById.get(compiled.target.pageId);

    const sourceDirectory = sourcePage
      ? normalizedSourceDirectory(sourcePage.sourcePath)
      : undefined;
    const relative = sourceDirectory === undefined
      ? undefined
      : normalizeSourceReference(sourceDirectory ? `${sourceDirectory}/${target}` : target);
    if (relative !== undefined) {
      const exactRelative = pagesBySource.get(relative);
      if (exactRelative) return uniquePage(exactRelative);
    }

    const normalized = normalizeSourceReference(target);
    if (normalized !== undefined && normalized !== relative) {
      const exactSource = pagesBySource.get(normalized);
      if (exactSource) return uniquePage(exactSource);
    }
    if (relative === undefined && normalized === undefined) return undefined;

    const aliases = pagesByAlias.get(normalizeAlias(target));
    if (aliases) return uniquePage(aliases);

    const basenameSource = relative ?? normalized;
    if (basenameSource === undefined) return undefined;
    const basename = basenameSource.slice(basenameSource.lastIndexOf("/") + 1);
    return uniquePage(pagesByBasename.get(basename) ?? []);
  };

  const childPages = (page: PageRecord, source?: string): readonly PageRecord[] => {
    const owner = source && source !== "children"
      ? resolvePageReference(source, page)
      : page;
    if (!owner) return [];
    return owner.children
      .map(getPageById)
      .filter((child) => child.status === "published");
  };

  const sortPages = (
    input: readonly PageRecord[],
    key: string,
    direction?: "asc" | "desc" | string,
  ): readonly PageRecord[] => {
    const sorted = [...input];
    const multiplier = direction === "desc" || (!direction && key === "date") ? -1 : 1;
    sorted.sort((left, right) => {
      const result = comparePageValue(left, right, key);
      return result * multiplier || left.route.localeCompare(right.route, "en");
    });
    return sorted;
  };

  const resolvedLink = (
    page: PageRecord,
    destination: string,
    span?: SourceSpan,
  ): ResolvedLink | undefined => page.outgoingLinks.find(
      (candidate) =>
        candidate.authoredDestination === destination &&
        (!span || sameSourceSpan(candidate.span, span)),
    );

  const resolvedEmbed = (
    page: PageRecord,
    destination: string,
    span?: SourceSpan,
  ): ResolvedEmbed | undefined => page.embeds.find(
    (candidate) =>
      candidate.authoredDestination === destination &&
      (!span || sameSourceSpan(candidate.span, span)),
  );

  const resolvedHref = (
    page: PageRecord,
    destination: string,
    span?: SourceSpan,
  ): string => {
    const link = resolvedLink(page, destination, span);
    if (!link) return destination;
    if (link.target.kind === "external") return link.target.href;
    if (link.target.kind === "page") {
      const fragment = link.target.fragment?.id;
      return fragment ? `${link.target.route}#${fragment}` : link.target.route;
    }
    return destination;
  };

  const entryPage = getPageById(immutableManifest.entryPage);
  return Object.freeze({
    manifest: immutableManifest,
    pages: immutablePages,
    entryPage,
    getPageById,
    getPageByRoute,
    resolvePageReference,
    childPages,
    sortPages,
    resolvedLink,
    resolvedEmbed,
    resolvedHref,
  });
}

export function normalizeRoute(route: string, trailingSlash = true): string {
  const pathname = route.split(/[?#]/, 1)[0] ?? "";
  const path = `/${pathname.split("/").filter(Boolean).join("/")}`;
  if (path === "/") return path;
  return trailingSlash ? `${path}/` : path;
}

export function segmentsFromRoute(route: string): readonly string[] {
  return normalizeRoute(route).split("/").filter(Boolean);
}

export function routeFromSegments(
  segments: readonly string[] = [],
  trailingSlash = true,
): string {
  return normalizeRoute(segments.join("/"), trailingSlash);
}

export function pageValue(page: PageRecord, key: string): JsonValue | undefined {
  if (key === "title") return page.title;
  if (key === "description") return page.description;
  if (key === "date") return page.date;
  if (key === "updated") return page.updated;
  if (key === "tags") return page.tags;
  return page.data[key];
}

export function objectValue(value: JsonValue | undefined): JsonObject | undefined {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    ? value as JsonObject
    : undefined;
}

export function stripWebTag(tag: string): string {
  return tag.replace(/^web\//, "");
}

export function formatDate(value: string, locale = "en-GB"): string {
  const date = new Date(`${value}T00:00:00Z`);
  if (Number.isNaN(date.valueOf())) return value;
  return new Intl.DateTimeFormat(locale, {
    day: "2-digit",
    month: "long",
    year: "numeric",
    timeZone: "UTC",
  }).format(date);
}

export function sameSourceSpan(
  left: SourceSpan | undefined,
  right: SourceSpan | undefined,
): boolean {
  if (!left || !right) return false;
  if (
    left.startByte !== undefined &&
    left.endByte !== undefined &&
    right.startByte !== undefined &&
    right.endByte !== undefined
  ) {
    return left.startByte === right.startByte && left.endByte === right.endByte;
  }
  return left.start.line === right.start.line &&
    left.start.column === right.start.column &&
    left.end.line === right.end.line &&
    left.end.column === right.end.column;
}

function normalizeSourceReference(value: string): string | undefined {
  const normalized = normalizeLogicalPath(value);
  return normalized
    ?.replace(/\.md$/i, "")
    .replace(/\/index$/i, "")
    .toLocaleLowerCase("en");
}

function normalizedSourceDirectory(value: string): string | undefined {
  const normalized = normalizeLogicalPath(value);
  if (normalized === undefined) return undefined;
  const separator = normalized.lastIndexOf("/");
  return separator < 0 ? "" : normalized.slice(0, separator);
}

function normalizeLogicalPath(value: string): string | undefined {
  const segments: string[] = [];
  for (const segment of value.replace(/\\/g, "/").split("/")) {
    if (!segment || segment === ".") continue;
    if (segment === "..") {
      if (segments.length === 0) return undefined;
      segments.pop();
    } else {
      segments.push(segment);
    }
  }
  return segments.join("/");
}

function normalizeAlias(value: string): string {
  return value.trim().toLocaleLowerCase("en");
}

function indexPages(
  pages: readonly PageRecord[],
  keyFor: (page: PageRecord) => string | undefined,
): Map<string, PageRecord[]> {
  const index = new Map<string, PageRecord[]>();
  for (const page of pages) addIndexedPage(index, keyFor(page), page);
  return index;
}

function addIndexedPage(
  index: Map<string, PageRecord[]>,
  key: string | undefined,
  page: PageRecord,
): void {
  if (!key) return;
  const entries = index.get(key);
  if (entries) entries.push(page);
  else index.set(key, [page]);
}

function uniquePage(pages: readonly PageRecord[]): PageRecord | undefined {
  return pages.length === 1 ? pages[0] : undefined;
}

function freezeValue<T>(value: T, seen: WeakSet<object>): T {
  if (value === null || (typeof value !== "object" && typeof value !== "function")) return value;
  const object = value as object;
  if (seen.has(object)) return value;
  seen.add(object);
  for (const key of Reflect.ownKeys(object)) {
    const descriptor = Object.getOwnPropertyDescriptor(object, key);
    if (descriptor && "value" in descriptor) freezeValue(descriptor.value, seen);
  }
  return Object.freeze(value);
}

function comparePageValue(left: PageRecord, right: PageRecord, key: string): number {
  if (key === "order") {
    return (left.order ?? Number.MAX_SAFE_INTEGER) -
      (right.order ?? Number.MAX_SAFE_INTEGER);
  }
  const leftValue = key === "path" ? left.route : pageValue(left, key);
  const rightValue = key === "path" ? right.route : pageValue(right, key);
  return String(leftValue ?? "").localeCompare(String(rightValue ?? ""), "en", {
    numeric: true,
  });
}

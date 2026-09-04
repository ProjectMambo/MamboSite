import {
  Fragment,
  createElement,
  type ComponentType,
  type ReactNode,
} from "react";
import {
  normalizeRoute,
  resolvedDirectiveProperties,
  routeFromSegments,
  sameSourceSpan,
  segmentsFromRoute,
  validatedDirectiveForNode,
  type DirectiveNode,
  type MarkdownNode,
  type PageRecord,
} from "@mambosite/runtime";
import { directiveConfig } from "./config.js";
import type {
  BacklinksDirectiveConfig,
  BreadcrumbItem,
  BreadcrumbsDirectiveConfig,
  ButtonDirectiveConfig,
  ChildrenDirectiveConfig,
  DirectiveComponentProps,
  DirectiveConfigMap,
  DirectiveModelMap,
  DirectiveName,
  EmbedNodeModel,
  GalleryDirectiveConfig,
  IncludeDirectiveConfig,
  MamboRuntime,
  NodeComponentProps,
  PageDirectiveConfig,
  RelatedDirectiveConfig,
  RenderableNodeName,
  RenderState,
  TocDirectiveConfig,
} from "./types.js";

const DIRECTIVE_NAMES = new Set<DirectiveName>([
  "page",
  "hero",
  "breadcrumbs",
  "meta",
  "toc",
  "children",
  "related",
  "backlinks",
  "gallery",
  "include",
  "button",
  "section",
  "columns",
  "column",
]);

export interface MarkdownRendererProps {
  readonly runtime: MamboRuntime;
  readonly page: PageRecord;
  readonly nodes?: readonly MarkdownNode[];
  readonly state?: RenderState;
}

export function MarkdownRenderer({
  runtime,
  page,
  nodes = page.body.children ?? [],
  state = initialRenderState(page),
}: MarkdownRendererProps) {
  return (
    <>
      {nodes.map((node, index) => (
        <Fragment key={nodeKey(node, index)}>
          {renderNode(runtime, page, node, state)}
        </Fragment>
      ))}
    </>
  );
}

export interface MamboPageProps {
  readonly runtime: MamboRuntime;
  readonly page: PageRecord;
}

export function MamboPage({ runtime, page }: MamboPageProps) {
  const state = initialRenderState(page);
  const pageNode = topLevelNodes(page).find(isPageDirective);
  const config = pageNode
    ? directiveConfig("page", resolvedDirectiveProperties(page, pageNode))
    : defaultPageConfig();
  const Layout = runtime.registry.layouts[config.layout] ?? runtime.registry.fallbacks.Layout;
  const body = <MarkdownRenderer page={page} runtime={runtime} state={state} />;
  const showGeneratedTitle = !topLevelNodes(page).some(
    (node) =>
      (node.type === "heading" && node.level === 1) ||
      heroShowsTitle(page, node),
  );
  const tocConfig: TocDirectiveConfig = {
    minDepth: 2,
    maxDepth: 4,
    ordered: true,
    title: "On this page",
    collapse: false,
  };
  const Toc = runtime.registry.directives.toc;
  const visibleHeadings = page.headings.filter(
    (heading) => heading.level >= tocConfig.minDepth && heading.level <= tocConfig.maxDepth,
  );
  const hasAuthoredToc = page.directives.some((directive) => directive.name === "toc");
  const sidebar = config.sidebar && !hasAuthoredToc && visibleHeadings.length > 0 ? (
    <Toc
      config={tocConfig}
      model={{ headings: visibleHeadings }}
      name="toc"
      page={page}
      runtime={runtime}
      state={state}
    >
      {null}
    </Toc>
  ) : null;
  const parentHref = page.route === "/"
    ? undefined
    : parentRoute(page.route, runtime.store.manifest.site.trailingSlash);

  return (
    <Layout
      config={config}
      page={page}
      {...(parentHref ? { parentHref } : {})}
      runtime={runtime}
      showGeneratedTitle={showGeneratedTitle}
      sidebar={sidebar}
    >
      {body}
    </Layout>
  );
}

export interface MamboSiteFrameProps {
  readonly runtime: MamboRuntime;
  readonly children: ReactNode;
}

export function MamboSiteFrame({ runtime, children }: MamboSiteFrameProps) {
  const { Header, Footer } = runtime.registry.shell;
  return (
    <>
      <Header runtime={runtime} />
      <main className="mambo-site-main">{children}</main>
      <Footer runtime={runtime} />
    </>
  );
}

export function MamboNotFound({ runtime }: { readonly runtime: MamboRuntime }) {
  const NotFound = runtime.registry.shell.NotFound;
  return <NotFound runtime={runtime} />;
}

function renderNode(
  runtime: MamboRuntime,
  page: PageRecord,
  node: MarkdownNode,
  state: RenderState,
): ReactNode {
  if (node.type === "directive") {
    return renderDirective(runtime, page, node, state);
  }

  const name = node.type as RenderableNodeName;
  const Component = runtime.registry.nodes[name] as
    | ComponentType<NodeComponentProps>
    | undefined;
  if (!Component) {
    const Fallback = runtime.registry.fallbacks.Node;
    return (
      <Fallback node={node} page={page}>
        <MarkdownRenderer nodes={node.children ?? []} page={page} runtime={runtime} state={state} />
      </Fallback>
    );
  }

  const childState = node.type === "tableRow"
    ? { ...state, tableHeader: node.header }
    : state;
  const children = (
    <MarkdownRenderer
      nodes={node.children ?? []}
      page={page}
      runtime={runtime}
      state={childState}
    />
  );
  const model = resolveNodeModel(runtime, page, node, state);
  return createElement(Component, {
    node,
    page,
    children,
    model,
    runtime,
    state,
  } as NodeComponentProps);
}

function renderDirective(
  runtime: MamboRuntime,
  page: PageRecord,
  node: MarkdownNode & DirectiveNode,
  state: RenderState,
): ReactNode {
  const authoredChildren = (
    <MarkdownRenderer
      nodes={node.children ?? []}
      page={page}
      runtime={runtime}
      state={state}
    />
  );
  if (!isDirectiveName(node.invocation.name)) {
    const Fallback = runtime.registry.fallbacks.Directive;
    return (
      <Fallback name={node.invocation.name} page={page}>
        {authoredChildren}
      </Fallback>
    );
  }

  const name = node.invocation.name;
  const config = directiveConfig(name, resolvedDirectiveProperties(page, node));
  const model = resolveDirectiveModel(runtime, page, node, name, config, state);
  const Component = runtime.registry.directives[name] as ComponentType<DirectiveComponentProps>;
  return createElement(Component, {
    name,
    page,
    config,
    model,
    children: authoredChildren,
    runtime,
    state,
  } as DirectiveComponentProps);
}

function resolveNodeModel(
  runtime: MamboRuntime,
  page: PageRecord,
  node: MarkdownNode,
  state: RenderState,
): object {
  let model: object = {};
  switch (node.type) {
    case "heading": {
      const heading = page.headings.find((candidate) => sameSourceSpan(candidate.span, node.span));
      const id = heading?.id ?? node.blockId;
      model = {
        ...(id ? { id: `${state.idPrefix}${id}` } : {}),
        level: Math.min(6, Math.max(1, node.level + state.headingOffset)),
      };
      break;
    }
    case "link":
      model = {
        href: embeddedFragmentHref(
          runtime.store.resolvedHref(page, node.destination, node.span),
          page,
          runtime,
          state,
        ),
      };
      break;
    case "wikiLink": {
      model = {
        href: embeddedFragmentHref(
          runtime.store.resolvedHref(page, node.destination, node.span),
          page,
          runtime,
          state,
        ),
        label: plainNodeText(node) || node.destination,
      };
      break;
    }
    case "image":
      model = { alt: plainNodeText(node) };
      break;
    case "obsidianEmbed":
      model = resolveEmbedModel(runtime, page, node, state);
      break;
    case "tableCell":
      model = { header: state.tableHeader };
      break;
  }
  return model;
}

function resolveEmbedModel(
  runtime: MamboRuntime,
  page: PageRecord,
  node: Extract<MarkdownNode, { readonly type: "obsidianEmbed" }>,
  state: RenderState,
): EmbedNodeModel {
  const target = runtime.store.resolvePageReference(node.destination, page);
  const resolved = runtime.store.resolvedEmbed(page, node.destination, node.span);
  const instanceId = resolved?.instanceId ?? fallbackInstanceId(page, node);
  const resolvedTarget = resolved?.target.kind === "page"
    ? runtime.store.getPageById(resolved.target.pageId)
    : target;
  if (!resolvedTarget) {
    return { kind: "asset", source: node.destination, instanceId };
  }
  const fragment = resolved?.target.kind === "page" ? resolved.target.fragment : undefined;
  if (fragment) {
    return {
      kind: "page",
      source: node.destination,
      instanceId,
      target: resolvedTarget,
      fragment,
      sourceHref: `${resolvedTarget.route}#${fragment.id}`,
      error: "fragment",
    };
  }
  if (state.embedded.has(resolvedTarget.id)) {
    return {
      kind: "page",
      source: node.destination,
      instanceId,
      target: resolvedTarget,
      error: "recursive",
    };
  }
  const embedded = new Set(state.embedded);
  embedded.add(resolvedTarget.id);
  return {
    kind: "page",
    source: node.destination,
    instanceId,
    target: resolvedTarget,
    content: (
      <MarkdownRenderer
        page={resolvedTarget}
        runtime={runtime}
        state={{
          ...state,
          embedded,
          headingOffset: state.headingOffset + 1,
          idPrefix: `${state.idPrefix}${instanceId}-`,
        }}
      />
    ),
  };
}

function resolveDirectiveModel<K extends DirectiveName>(
  runtime: MamboRuntime,
  page: PageRecord,
  node: MarkdownNode & DirectiveNode,
  name: K,
  config: DirectiveConfigMap[K],
  state: RenderState,
): DirectiveModelMap[K] {
  let model: object;
  switch (name) {
    case "breadcrumbs":
      model = breadcrumbsModel(runtime, page, config as BreadcrumbsDirectiveConfig);
      break;
    case "toc": {
      const toc = config as TocDirectiveConfig;
      model = {
        headings: page.headings.filter(
          (heading) => heading.level >= toc.minDepth && heading.level <= toc.maxDepth,
        ).map((heading) => ({
          ...heading,
          id: `${state.idPrefix}${heading.id}`,
        })),
      };
      break;
    }
    case "children":
      model = childrenModel(runtime, page, config as ChildrenDirectiveConfig);
      break;
    case "related":
      model = relatedModel(runtime, page, config as RelatedDirectiveConfig);
      break;
    case "backlinks":
      model = backlinksModel(runtime, page, config as BacklinksDirectiveConfig);
      break;
    case "gallery":
      model = galleryModel(runtime, page, config as GalleryDirectiveConfig);
      break;
    case "include":
      model = includeModel(runtime, page, node, config as IncludeDirectiveConfig, state);
      break;
    case "button": {
      const button = config as ButtonDirectiveConfig;
      const href = button.external
          ? button.href
          : embeddedFragmentHref(
            runtime.store.resolvedHref(page, button.href),
            page,
            runtime,
            state,
          );
      model = safeLinkHref(href)
        ? { href }
        : { href: "#", error: "unsafe" };
      break;
    }
    default:
      model = {};
  }
  return model as DirectiveModelMap[K];
}

function breadcrumbsModel(
  runtime: MamboRuntime,
  page: PageRecord,
  config: BreadcrumbsDirectiveConfig,
): { readonly items: readonly BreadcrumbItem[] } {
  const segments = segmentsFromRoute(page.route);
  const items: BreadcrumbItem[] = [
    { label: config.home, href: "/", current: segments.length === 0 },
  ];
  segments.forEach((segment, index) => {
    const current = index === segments.length - 1;
    if (current && !config.includeCurrent) return;
    const href = routeFromSegments(
      segments.slice(0, index + 1),
      runtime.store.manifest.site.trailingSlash,
    );
    const target = runtime.store.getPageByRoute(href);
    items.push({
      label: target?.title ?? segment.replaceAll("-", " "),
      href,
      current,
    });
  });
  return { items };
}

function childrenModel(
  runtime: MamboRuntime,
  page: PageRecord,
  config: ChildrenDirectiveConfig,
) {
  const available = runtime.store.childPages(page, config.source).filter(
    (child) => child.listed || config.includeUnlisted,
  );
  const sorted = runtime.store.sortPages(available, config.sort, config.direction);
  const items = config.limit === undefined ? sorted : sorted.slice(0, config.limit);
  return {
    items,
    emptyMessage: config.empty === "message" ? "Nothing here yet." : "",
  };
}

function relatedModel(
  runtime: MamboRuntime,
  page: PageRecord,
  config: RelatedDirectiveConfig,
) {
  const linkedIds = new Set(
    page.outgoingLinks.flatMap((link) =>
      link.target.kind === "page" ? [link.target.pageId] : []),
  );
  const ranked = runtime.store.pages
    .filter(
      (candidate) =>
        candidate.id !== page.id &&
        candidate.status === "published" &&
        (candidate.listed || config.includeUnlisted),
    )
    .map((candidate) => {
      const tagScore = candidate.tags.filter((tag) => page.tags.includes(tag)).length;
      const linkScore = linkedIds.has(candidate.id) || candidate.backlinks.includes(page.id) ? 1 : 0;
      const score = config.by === "tags"
        ? tagScore
        : config.by === "links"
          ? linkScore
          : tagScore + linkScore;
      return { candidate, score };
    })
    .filter(({ score }) => score > 0)
    .sort(
      (left, right) =>
        right.score - left.score || left.candidate.route.localeCompare(right.candidate.route, "en"),
    )
    .slice(0, config.limit)
    .map(({ candidate }) => candidate);
  return { items: ranked, emptyMessage: "" };
}

function backlinksModel(
  runtime: MamboRuntime,
  page: PageRecord,
  config: BacklinksDirectiveConfig,
) {
  const available = page.backlinks
    .map((id) => runtime.store.getPageById(id))
    .filter((candidate) => candidate.status === "published");
  const items = config.limit === undefined ? available : available.slice(0, config.limit);
  return { items, emptyMessage: config.empty };
}

function galleryModel(
  runtime: MamboRuntime,
  page: PageRecord,
  config: GalleryDirectiveConfig,
) {
  const items = runtime.store.sortPages(
    runtime.store.childPages(page, config.source),
    "order",
    "asc",
  );
  return { items, emptyMessage: "" };
}

function includeModel(
  runtime: MamboRuntime,
  page: PageRecord,
  node: MarkdownNode & DirectiveNode,
  config: IncludeDirectiveConfig,
  state: RenderState,
) {
  if (config.source.includes("#")) return { error: "fragment" as const };
  const target = runtime.store.resolvePageReference(config.source, page);
  if (!target) return { error: "missing" as const };
  if (state.embedded.has(target.id)) {
    return { target, sourceHref: target.route, error: "recursive" as const };
  }
  const embedded = new Set(state.embedded);
  embedded.add(target.id);
  const instanceId = fallbackInstanceId(page, node);
  const headingOffset = config.headings === "shift" ? state.headingOffset + 1 : state.headingOffset;
  const nodes = config.headings === "strip-title"
    ? topLevelNodes(target).filter((node) => !(node.type === "heading" && node.level === 1))
    : topLevelNodes(target);
  return {
    target,
    instanceId,
    sourceHref: target.route,
    content: (
      <MarkdownRenderer
        nodes={nodes}
        page={target}
        runtime={runtime}
        state={{
          ...state,
          embedded,
          headingOffset,
          idPrefix: `${state.idPrefix}${instanceId}-`,
        }}
      />
    ),
  };
}

function topLevelNodes(page: PageRecord): readonly MarkdownNode[] {
  return page.body.children ?? [];
}

function isPageDirective(
  node: MarkdownNode,
): node is MarkdownNode & DirectiveNode {
  return node.type === "directive" && node.invocation.name === "page";
}

function isDirectiveName(name: string): name is DirectiveName {
  return DIRECTIVE_NAMES.has(name as DirectiveName);
}

function heroShowsTitle(page: PageRecord, node: MarkdownNode): boolean {
  if (node.type !== "directive" || node.invocation.name !== "hero") return false;
  if (!validatedDirectiveForNode(page, node)) return false;
  return directiveConfig("hero", resolvedDirectiveProperties(page, node)).showTitle;
}

function defaultPageConfig(): PageDirectiveConfig {
  return { layout: "default", width: "normal", sidebar: true };
}

function initialRenderState(page: PageRecord): RenderState {
  return {
    headingOffset: 0,
    idPrefix: "",
    embedded: new Set([page.id]),
    tableHeader: false,
  };
}

function fallbackInstanceId(page: PageRecord, node: MarkdownNode): string {
  const location = node.span?.startByte ?? node.span?.start.line ?? 0;
  return `e_${page.id}_${location}`.replace(/[^A-Za-z0-9_-]/g, "_");
}

function safeLinkHref(value: string): boolean {
  const href = value.trim();
  if (!href || /[\u0000-\u001f\u007f]/.test(href)) return false;
  if (href.startsWith("/") || href.startsWith("#") || href.startsWith("./") || href.startsWith("../")) {
    return true;
  }
  const scheme = /^([A-Za-z][A-Za-z0-9+.-]*):/.exec(href)?.[1]?.toLowerCase();
  return scheme === undefined || ["http", "https", "mailto", "tel"].includes(scheme);
}

function embeddedFragmentHref(
  href: string,
  page: PageRecord,
  runtime: MamboRuntime,
  state: RenderState,
): string {
  if (!state.idPrefix) return href;
  const hashIndex = href.indexOf("#");
  if (hashIndex < 0 || hashIndex === href.length - 1) return href;

  const target = href.slice(0, hashIndex);
  if (/^[A-Za-z][A-Za-z0-9+.-]*:/.test(target) || target.startsWith("//")) return href;
  const targetPath = target.split("?", 1)[0] ?? "";
  const sameTarget = targetPath === "" || normalizeRoute(
    targetPath,
    runtime.store.manifest.site.trailingSlash,
  ) === normalizeRoute(page.route, runtime.store.manifest.site.trailingSlash);
  if (!sameTarget) return href;

  const fragment = href.slice(hashIndex + 1);
  return `#${fragment.startsWith(state.idPrefix) ? fragment : `${state.idPrefix}${fragment}`}`;
}

function parentRoute(route: string, trailingSlash: boolean): string {
  const segments = segmentsFromRoute(route);
  if (segments.length <= 1) return "/";
  return routeFromSegments(segments.slice(0, -1), trailingSlash);
}

function plainNodeText(node: MarkdownNode): string {
  if (node.type === "text") return node.value;
  return (node.children ?? []).map(plainNodeText).join("");
}

function nodeKey(node: MarkdownNode, index: number): string {
  return `${node.type}-${node.span?.startByte ?? `${node.span?.start.line ?? 0}-${index}`}`;
}

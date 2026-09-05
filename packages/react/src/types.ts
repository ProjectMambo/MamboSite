import type { ComponentType, ReactNode } from "react";
import type {
  ContentStore,
  HeadingRecord,
  MarkdownNode,
  PageRecord,
  ResolvedFragment,
} from "@mambosite/runtime";

export type DirectiveName =
  | "page"
  | "hero"
  | "breadcrumbs"
  | "meta"
  | "toc"
  | "timestamp"
  | "children"
  | "related"
  | "backlinks"
  | "gallery"
  | "include"
  | "button"
  | "section"
  | "columns"
  | "column"
  | "footer";

export type LayoutName =
  | "default"
  | "article"
  | "docs"
  | "project"
  | "collection"
  | "home"
  | "gallery";

export type PageWidth = "narrow" | "normal" | "wide" | "full";

export interface PageDirectiveConfig {
  readonly layout: LayoutName;
  readonly width: PageWidth;
  readonly sidebar: boolean;
}

export interface HeroDirectiveConfig {
  readonly image?: string;
  readonly align: "left" | "center" | "split";
  readonly showTitle: boolean;
  readonly showDescription: boolean;
  readonly showMeta: boolean;
}

export interface BreadcrumbsDirectiveConfig {
  readonly home: string;
  readonly separator: string;
  readonly includeCurrent: boolean;
}

export interface MetaDirectiveConfig {
  readonly show: readonly string[];
  readonly style: "inline" | "stack" | "table";
  readonly empty: "hide" | "placeholder";
}

export interface TocDirectiveConfig {
  readonly minDepth: number;
  readonly maxDepth: number;
  readonly ordered: boolean;
  readonly title: string;
  readonly collapse: boolean;
}

export interface TimestampDirectiveConfig {
  readonly timezone: string;
  readonly label: string;
}

export interface ChildrenDirectiveConfig {
  readonly source?: string;
  readonly view: "list" | "grid" | "cards" | "tree" | "table" | "hidden";
  readonly depth: number;
  readonly sort: "order" | "title" | "date" | "updated" | "path";
  readonly direction?: "asc" | "desc";
  readonly columns: number;
  readonly limit?: number;
  readonly show: readonly string[];
  readonly includeUnlisted: boolean;
  readonly empty: "hide" | "message";
}

export interface RelatedDirectiveConfig {
  readonly by: "tags" | "links" | "both";
  readonly view: string;
  readonly limit: number;
  readonly show: readonly string[];
  readonly includeUnlisted: boolean;
}

export interface BacklinksDirectiveConfig {
  readonly view: "list" | "cards";
  readonly limit?: number;
  readonly show: readonly string[];
  readonly empty: string;
}

export interface GalleryDirectiveConfig {
  readonly source: string;
  readonly view: "grid" | "masonry" | "carousel";
  readonly columns: number;
  readonly fit: "cover" | "contain" | "natural";
  readonly captions: boolean;
}

export interface IncludeDirectiveConfig {
  readonly source: string;
  readonly mode: "embed" | "inline";
  readonly headings: "shift" | "keep" | "strip-title";
  readonly showTitle: boolean;
  readonly showSource: boolean;
}

export interface ButtonDirectiveConfig {
  readonly label: string;
  readonly href: string;
  readonly variant: "primary" | "secondary" | "quiet" | "card";
  readonly external: boolean;
  readonly icon?: string;
}

export interface SectionDirectiveConfig {
  readonly width: PageWidth;
  readonly tone: "plain" | "subtle" | "brand" | "success" | "warning" | "danger";
  readonly align: "left" | "center" | "right";
  readonly id?: string;
}

export interface ColumnsDirectiveConfig {
  readonly count: number;
  readonly gap: "small" | "normal" | "large";
  readonly collapseAt: "sm" | "md" | "lg" | "never";
}

export interface ColumnDirectiveConfig {}
export interface FooterDirectiveConfig {}

export interface DirectiveConfigMap {
  readonly page: PageDirectiveConfig;
  readonly hero: HeroDirectiveConfig;
  readonly breadcrumbs: BreadcrumbsDirectiveConfig;
  readonly meta: MetaDirectiveConfig;
  readonly toc: TocDirectiveConfig;
  readonly timestamp: TimestampDirectiveConfig;
  readonly children: ChildrenDirectiveConfig;
  readonly related: RelatedDirectiveConfig;
  readonly backlinks: BacklinksDirectiveConfig;
  readonly gallery: GalleryDirectiveConfig;
  readonly include: IncludeDirectiveConfig;
  readonly button: ButtonDirectiveConfig;
  readonly section: SectionDirectiveConfig;
  readonly columns: ColumnsDirectiveConfig;
  readonly column: ColumnDirectiveConfig;
  readonly footer: FooterDirectiveConfig;
}

export interface EmptyModel {}

export interface CollectionModel {
  readonly items: readonly PageRecord[];
  readonly emptyMessage: string;
}

export interface BreadcrumbItem {
  readonly label: string;
  readonly href: string;
  readonly current: boolean;
}

export interface BreadcrumbsModel {
  readonly items: readonly BreadcrumbItem[];
}

export interface TocModel {
  readonly headings: readonly HeadingRecord[];
}

export interface IncludeModel {
  readonly instanceId?: string;
  readonly target?: PageRecord;
  readonly content?: ReactNode;
  readonly sourceHref?: string;
  readonly error?: "missing" | "recursive" | "fragment";
}

export interface ButtonModel {
  readonly href: string;
  readonly error?: "unsafe";
}

export interface DirectiveModelMap {
  readonly page: EmptyModel;
  readonly hero: EmptyModel;
  readonly breadcrumbs: BreadcrumbsModel;
  readonly meta: EmptyModel;
  readonly toc: TocModel;
  readonly timestamp: EmptyModel;
  readonly children: CollectionModel;
  readonly related: CollectionModel;
  readonly backlinks: CollectionModel;
  readonly gallery: CollectionModel;
  readonly include: IncludeModel;
  readonly button: ButtonModel;
  readonly section: EmptyModel;
  readonly columns: EmptyModel;
  readonly column: EmptyModel;
  readonly footer: EmptyModel;
}

export interface RenderState {
  readonly headingOffset: number;
  readonly idPrefix: string;
  readonly embedded: ReadonlySet<string>;
  readonly tableHeader: boolean;
}

export interface HeadingNodeModel {
  readonly id?: string;
  readonly level: number;
}

export interface LinkNodeModel {
  readonly href: string;
}

export interface WikiLinkNodeModel extends LinkNodeModel {
  readonly label: string;
}

export interface ImageNodeModel {
  readonly alt: string;
}

export interface EmbedNodeModel {
  readonly kind: "page" | "asset";
  readonly source: string;
  readonly instanceId: string;
  readonly fragment?: ResolvedFragment;
  readonly sourceHref?: string;
  readonly target?: PageRecord;
  readonly content?: ReactNode;
  readonly error?: "recursive" | "fragment";
}

export interface TableCellNodeModel {
  readonly header: boolean;
}

export interface NodeModelOverrides {
  readonly heading: HeadingNodeModel;
  readonly link: LinkNodeModel;
  readonly wikiLink: WikiLinkNodeModel;
  readonly image: ImageNodeModel;
  readonly obsidianEmbed: EmbedNodeModel;
  readonly tableCell: TableCellNodeModel;
}

export type NodeModel<K extends MarkdownNode["type"]> =
  K extends keyof NodeModelOverrides ? NodeModelOverrides[K] : EmptyModel;

export type NodeOf<K extends MarkdownNode["type"]> = Extract<MarkdownNode, { readonly type: K }>;
export type RenderableNodeName = Exclude<MarkdownNode["type"], "directive">;

export interface MamboRuntimeOptions {
  readonly locale?: string;
  readonly theme?: {
    readonly defaultScheme: string;
    readonly schemes: readonly string[];
  };
}

export interface MamboRuntime {
  readonly store: ContentStore;
  readonly registry: MamboComponentRegistry;
  readonly options: Readonly<MamboRuntimeOptions>;
}

export interface NodeComponentProps<K extends RenderableNodeName = RenderableNodeName> {
  readonly node: NodeOf<K>;
  readonly page: PageRecord;
  readonly children: ReactNode;
  readonly model: NodeModel<K>;
  readonly runtime: MamboRuntime;
  readonly state: RenderState;
}

export interface DirectiveComponentProps<K extends DirectiveName = DirectiveName> {
  readonly name: K;
  readonly page: PageRecord;
  readonly config: DirectiveConfigMap[K];
  readonly model: DirectiveModelMap[K];
  readonly children: ReactNode;
  readonly runtime: MamboRuntime;
  readonly state: RenderState;
}

export interface LinkPrimitiveProps {
  readonly href: string;
  readonly children: ReactNode;
  readonly accentItem?: boolean;
  readonly className?: string;
  readonly title?: string;
  readonly newTab?: boolean;
  readonly current?: boolean;
}

export interface ImagePrimitiveProps {
  readonly src: string;
  readonly alt: string;
  readonly className?: string;
  readonly title?: string;
  readonly decorative?: boolean;
}

export interface PrimitiveRegistry {
  readonly Link: ComponentType<LinkPrimitiveProps>;
  readonly Image: ComponentType<ImagePrimitiveProps>;
}

export type NodeRegistry = {
  readonly [K in RenderableNodeName]: ComponentType<NodeComponentProps<K>>;
};

export type DirectiveRegistry = {
  readonly [K in DirectiveName]: ComponentType<DirectiveComponentProps<K>>;
};

export interface PageLayoutProps {
  readonly page: PageRecord;
  readonly config: PageDirectiveConfig;
  readonly children: ReactNode;
  readonly sidebar: ReactNode;
  readonly showGeneratedTitle: boolean;
  readonly parentHref?: string;
  readonly runtime: MamboRuntime;
}

export type LayoutRegistry = {
  readonly [K in LayoutName]: ComponentType<PageLayoutProps>;
};

export interface SiteHeaderProps {
  readonly runtime: MamboRuntime;
}

export interface SiteFooterProps {
  readonly runtime: MamboRuntime;
}

export interface NotFoundProps {
  readonly runtime: MamboRuntime;
}

export interface ShellRegistry {
  readonly Header: ComponentType<SiteHeaderProps>;
  readonly Footer: ComponentType<SiteFooterProps>;
  readonly NotFound: ComponentType<NotFoundProps>;
}

export interface UnsupportedNodeProps {
  readonly node: MarkdownNode;
  readonly page: PageRecord;
  readonly children: ReactNode;
}

export interface UnsupportedDirectiveProps {
  readonly name: string;
  readonly page: PageRecord;
  readonly children: ReactNode;
}

export interface RegistryFallbacks {
  readonly Node: ComponentType<UnsupportedNodeProps>;
  readonly Directive: ComponentType<UnsupportedDirectiveProps>;
  readonly Layout: ComponentType<PageLayoutProps>;
}

export interface MamboComponentRegistry {
  readonly primitives: PrimitiveRegistry;
  readonly nodes: NodeRegistry;
  readonly directives: DirectiveRegistry;
  readonly layouts: LayoutRegistry;
  readonly shell: ShellRegistry;
  readonly fallbacks: RegistryFallbacks;
}

export interface RegistryOverrides {
  readonly primitives?: Partial<PrimitiveRegistry>;
  readonly nodes?: Partial<NodeRegistry>;
  readonly directives?: Partial<DirectiveRegistry>;
  readonly layouts?: Partial<LayoutRegistry>;
  readonly shell?: Partial<ShellRegistry>;
  readonly fallbacks?: Partial<RegistryFallbacks>;
}

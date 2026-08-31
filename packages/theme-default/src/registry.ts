import {
  defineRegistry,
  type DirectiveRegistry,
  type LayoutRegistry,
  type ShellRegistry,
} from "@mambosite/react";
import {
  BacklinkCollection,
  Breadcrumbs,
  Button,
  ChildrenCollection,
  Column,
  Columns,
  Gallery,
  Hero,
  HiddenDirective,
  Include,
  Metadata,
  RelatedCollection,
  Section,
  TableOfContents,
} from "./directives/index.js";
import { defaultFallbacks } from "./fallbacks.js";
import {
  ArticleLayout,
  CollectionLayout,
  DefaultLayout,
  DocsLayout,
  GalleryLayout,
  HomeLayout,
  ProjectLayout,
} from "./layouts/DefaultLayout.js";
import { defaultNodeRegistry } from "./nodes.js";
import { defaultPrimitives } from "./primitives.js";
import { NotFound } from "./shell/NotFound.js";
import { SiteFooter } from "./shell/SiteFooter.js";
import { SiteHeader } from "./shell/SiteHeader.js";

export const defaultDirectiveRegistry = Object.freeze({
  page: HiddenDirective,
  hero: Hero,
  breadcrumbs: Breadcrumbs,
  meta: Metadata,
  toc: TableOfContents,
  children: ChildrenCollection,
  related: RelatedCollection,
  backlinks: BacklinkCollection,
  gallery: Gallery,
  include: Include,
  button: Button,
  section: Section,
  columns: Columns,
  column: Column,
}) satisfies DirectiveRegistry;

export const defaultLayoutRegistry = Object.freeze({
  default: DefaultLayout,
  article: ArticleLayout,
  docs: DocsLayout,
  project: ProjectLayout,
  collection: CollectionLayout,
  home: HomeLayout,
  gallery: GalleryLayout,
}) satisfies LayoutRegistry;

export const defaultShellRegistry = Object.freeze({
  Header: SiteHeader,
  Footer: SiteFooter,
  NotFound,
}) satisfies ShellRegistry;

export const defaultRegistry = defineRegistry({
  primitives: defaultPrimitives,
  nodes: defaultNodeRegistry,
  directives: defaultDirectiveRegistry,
  layouts: defaultLayoutRegistry,
  shell: defaultShellRegistry,
  fallbacks: defaultFallbacks,
});

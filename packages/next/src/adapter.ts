import type { Metadata } from "next";
import {
  createMamboRuntime,
  createRegistry,
  type MamboComponentRegistry,
  type MamboRuntime,
  type MamboRuntimeOptions,
} from "@mambosite/react";
import {
  routeFromSegments,
  segmentsFromRoute,
  type PageRecord,
  type SiteManifest,
} from "@mambosite/runtime";
import { NextLink } from "./NextLink.js";
import { NextImage } from "./NextImage.js";

/** Replace only navigation; every content and theme component stays reusable. */
export function createNextRegistry(
  base: MamboComponentRegistry,
  basePath = "",
): MamboComponentRegistry {
  const StaticImage = (props: Parameters<typeof NextImage>[0]) => (
    NextImage({ ...props, basePath })
  );
  return createRegistry(base, {
    primitives: { Link: NextLink, Image: StaticImage },
  });
}

export interface CreateNextRuntimeInput {
  readonly manifest: SiteManifest;
  readonly pages: readonly PageRecord[];
  readonly registry: MamboComponentRegistry;
  readonly options?: MamboRuntimeOptions;
}

export function createNextRuntime({
  manifest,
  pages,
  registry,
  options,
}: CreateNextRuntimeInput): MamboRuntime {
  return createMamboRuntime({
    manifest,
    pages,
    registry: createNextRegistry(registry, manifest.site.basePath),
    ...(options ? { options } : {}),
  });
}

export function staticPageParams(
  runtime: MamboRuntime,
): readonly { readonly slug: readonly string[] }[] {
  return runtime.store.pages
    .filter((page) => page.status === "published" && page.route !== "/")
    .map((page) => ({ slug: segmentsFromRoute(page.route) }));
}

export function pageFromSegments(
  runtime: MamboRuntime,
  segments: readonly string[] = [],
): PageRecord | undefined {
  return runtime.store.getPageByRoute(
    routeFromSegments(segments, runtime.store.manifest.site.trailingSlash),
  );
}

export function metadataForPage(page: PageRecord | undefined): Metadata {
  if (!page) return {};
  return {
    title: page.title,
    ...(page.description ? { description: page.description } : {}),
  };
}

export function siteMetadata(runtime: MamboRuntime): Metadata {
  const { site } = runtime.store.manifest;
  return {
    title: {
      default: site.title,
      template: `%s | ${site.title}`,
    },
    ...(runtime.store.entryPage.description
      ? { description: runtime.store.entryPage.description }
      : {}),
    ...(site.url ? { metadataBase: new URL(site.url) } : {}),
  };
}

/** Inline before paint to avoid a colour-scheme flash during static hydration. */
export function themeBootstrapScript(defaultScheme: string): string {
  const fallback = JSON.stringify(defaultScheme)
    .replaceAll("<", "\\u003c")
    .replaceAll("\u2028", "\\u2028")
    .replaceAll("\u2029", "\\u2029");
  return `(function(){try{var t=localStorage.getItem('mambo-theme')||${fallback};document.documentElement.dataset.theme=t;}catch(e){document.documentElement.dataset.theme=${fallback};}})();`;
}

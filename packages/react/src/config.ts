import {
  booleanProperty,
  numberProperty,
  stringArrayProperty,
  stringProperty,
  type DirectiveProperties,
} from "@mambosite/runtime";
import type {
  DirectiveConfigMap,
  DirectiveName,
  LayoutName,
  PageWidth,
} from "./types.js";

export function directiveConfig<K extends DirectiveName>(
  name: K,
  properties: DirectiveProperties,
): DirectiveConfigMap[K] {
  return configByName(name, properties) as DirectiveConfigMap[K];
}

function configByName(
  name: DirectiveName,
  properties: DirectiveProperties,
): DirectiveConfigMap[DirectiveName] {
  switch (name) {
    case "page":
      return {
        layout: stringProperty(properties, "layout", "default") as LayoutName,
        width: stringProperty(properties, "width", "normal") as PageWidth,
        sidebar: booleanProperty(properties, "sidebar", true),
      };
    case "hero": {
      const image = stringProperty(properties, "image") || undefined;
      return {
        ...(image ? { image } : {}),
        align: stringProperty(properties, "align", "left") as "left" | "center" | "split",
        showTitle: booleanProperty(properties, "show-title", true),
        showDescription: booleanProperty(properties, "show-description", true),
        showMeta: booleanProperty(properties, "show-meta", false),
      };
    }
    case "breadcrumbs":
      return {
        home: stringProperty(properties, "home", "/"),
        separator: stringProperty(properties, "separator", "/"),
        includeCurrent: booleanProperty(properties, "include-current", true),
      };
    case "meta":
      return {
        show: stringArrayProperty(properties, "show"),
        style: stringProperty(properties, "style", "stack") as "inline" | "stack" | "table",
        empty: stringProperty(properties, "empty", "hide") as "hide" | "placeholder",
      };
    case "toc":
      return {
        minDepth: numberProperty(properties, "min-depth", 2),
        maxDepth: numberProperty(properties, "max-depth", 4),
        ordered: booleanProperty(properties, "ordered", true),
        title: stringProperty(properties, "title", "On this page"),
        collapse: booleanProperty(properties, "collapse", false),
      };
    case "timestamp":
      return {
        timezone: stringProperty(properties, "timezone", "UTC"),
        label: stringProperty(properties, "label", "Built"),
      };
    case "children": {
      const source = stringProperty(properties, "source") || undefined;
      const direction = stringProperty(properties, "direction") || undefined;
      const limit = optionalPositiveNumber(properties, "limit");
      return {
        ...(source ? { source } : {}),
        view: stringProperty(properties, "view", "list") as DirectiveConfigMap["children"]["view"],
        depth: numberProperty(properties, "depth", 1),
        sort: stringProperty(properties, "sort", "order") as DirectiveConfigMap["children"]["sort"],
        ...(direction ? { direction: direction as "asc" | "desc" } : {}),
        columns: numberProperty(properties, "columns", 3),
        ...(limit === undefined ? {} : { limit }),
        show: stringArrayProperty(properties, "show"),
        includeUnlisted: booleanProperty(properties, "include-unlisted", false),
        empty: stringProperty(properties, "empty", "hide") as "hide" | "message",
      };
    }
    case "related":
      return {
        by: stringProperty(properties, "by", "tags") as "tags" | "links" | "both",
        view: stringProperty(properties, "view", "cards"),
        limit: numberProperty(properties, "limit", 4),
        show: stringArrayProperty(properties, "show"),
        includeUnlisted: booleanProperty(properties, "include-unlisted", false),
      };
    case "backlinks": {
      const limit = optionalPositiveNumber(properties, "limit");
      return {
        view: stringProperty(properties, "view", "list") as "list" | "cards",
        ...(limit === undefined ? {} : { limit }),
        show: stringArrayProperty(properties, "show"),
        empty: stringProperty(properties, "empty", "Nothing links here yet."),
      };
    }
    case "gallery":
      return {
        source: stringProperty(properties, "source", "children"),
        view: stringProperty(properties, "view", "grid") as "grid" | "masonry" | "carousel",
        columns: numberProperty(properties, "columns", 3),
        fit: stringProperty(properties, "fit", "cover") as "cover" | "contain" | "natural",
        captions: booleanProperty(properties, "captions", true),
      };
    case "include":
      return {
        source: stringProperty(properties, "source"),
        mode: stringProperty(properties, "mode", "embed") as "embed" | "inline",
        headings: stringProperty(properties, "headings", "keep") as "shift" | "keep" | "strip-title",
        showTitle: booleanProperty(properties, "show-title", true),
        showSource: booleanProperty(properties, "show-source", false),
      };
    case "button": {
      const icon = stringProperty(properties, "icon") || undefined;
      return {
        label: stringProperty(properties, "label", "Open"),
        href: stringProperty(properties, "href", "#"),
        variant: stringProperty(properties, "variant", "primary") as "primary" | "secondary" | "quiet" | "card",
        external: booleanProperty(properties, "external", false),
        ...(icon ? { icon } : {}),
      };
    }
    case "section": {
      const id = stringProperty(properties, "id") || undefined;
      return {
        width: stringProperty(properties, "width", "normal") as PageWidth,
        tone: stringProperty(properties, "tone", "plain") as DirectiveConfigMap["section"]["tone"],
        align: stringProperty(properties, "align", "left") as "left" | "center" | "right",
        ...(id ? { id } : {}),
      };
    }
    case "columns":
      return {
        count: numberProperty(properties, "count", 2),
        gap: stringProperty(properties, "gap", "normal") as "small" | "normal" | "large",
        collapseAt: stringProperty(properties, "collapse-at", "md") as "sm" | "md" | "lg" | "never",
      };
    case "column":
    case "footer":
      return {};
  }
}

function optionalPositiveNumber(
  properties: DirectiveProperties,
  name: string,
): number | undefined {
  const value = properties[name];
  return typeof value === "number" && value > 0 ? value : undefined;
}

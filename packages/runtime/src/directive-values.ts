import type {
  DirectiveScalar,
  DirectiveValue,
  ParsedDirective,
  ValidatedDirective,
} from "./directive.js";
import type { DirectiveNode, MarkdownNode } from "./markdown.js";
import type { PageRecord } from "./page.js";
import { sameSourceSpan } from "./content-store.js";

export type DirectivePropertyValue =
  | string
  | number
  | boolean
  | readonly (string | number | boolean)[];

export type DirectiveProperties = Readonly<Record<string, DirectivePropertyValue>>;

export function directiveProperties(
  directive: ParsedDirective | ValidatedDirective,
): DirectiveProperties {
  const properties = Array.isArray(directive.properties)
    ? directive.properties.map((property) => [property.name, directiveValue(property.value)] as const)
    : Object.entries(directive.properties).map(
      ([name, value]) => [name, directiveValue(value)] as const,
    );
  return Object.freeze(Object.fromEntries(properties));
}

/**
 * Schema-1 compatibility bridge.
 *
 * The body tree retains parsed directive nodes while `page.directives` carries
 * validation defaults. Both records retain the directive source span, so the
 * runtime can use the validated record without reparsing Markdown. A later wire
 * schema can place this normalized data directly on the body node.
 */
export function validatedDirectiveForNode(
  page: PageRecord,
  node: MarkdownNode & DirectiveNode,
): ValidatedDirective | undefined {
  return page.directives.find(
    (candidate) =>
      candidate.name === node.invocation.name &&
      candidate.form === node.invocation.form &&
      sameSourceSpan(candidate.span, node.span),
  );
}

export function resolvedDirectiveProperties(
  page: PageRecord,
  node: MarkdownNode & DirectiveNode,
): DirectiveProperties {
  return directiveProperties(validatedDirectiveForNode(page, node) ?? node.invocation);
}

export function stringProperty(
  properties: DirectiveProperties,
  name: string,
  fallback = "",
): string {
  const value = properties[name];
  return typeof value === "string" ? value : fallback;
}

export function numberProperty(
  properties: DirectiveProperties,
  name: string,
  fallback: number,
): number {
  const value = properties[name];
  return typeof value === "number" ? value : fallback;
}

export function booleanProperty(
  properties: DirectiveProperties,
  name: string,
  fallback: boolean,
): boolean {
  const value = properties[name];
  return typeof value === "boolean" ? value : fallback;
}

export function stringArrayProperty(
  properties: DirectiveProperties,
  name: string,
): readonly string[] {
  const value = properties[name];
  if (!Array.isArray(value)) return [];
  return value.filter((item): item is string => typeof item === "string");
}

function directiveValue(value: DirectiveValue): DirectivePropertyValue {
  if (value.type === "array") return value.value.map(scalarValue);
  return scalarValue(value);
}

function scalarValue(value: DirectiveScalar): string | number | boolean {
  return value.value;
}

/** A half-open UTF-8 byte range in the authored Markdown source. */
export interface DirectiveSpan {
  readonly start: number;
  readonly end: number;
}

export type DirectiveForm = "leaf" | "container";

export type DirectiveScalar =
  | { readonly type: "string"; readonly value: string }
  | { readonly type: "number"; readonly value: number }
  | { readonly type: "boolean"; readonly value: boolean };

export type DirectiveValue = DirectiveScalar | {
  readonly type: "array";
  readonly value: readonly DirectiveScalar[];
};

export interface DirectiveProperty {
  readonly name: string;
  readonly value: DirectiveValue;
  readonly span: DirectiveSpan;
  readonly nameSpan: DirectiveSpan;
  readonly valueSpan: DirectiveSpan;
  readonly raw: string;
}

/** A syntactically valid directive before component-registry validation. */
export interface ParsedDirective {
  readonly form: DirectiveForm;
  readonly name: string;
  readonly properties: readonly DirectiveProperty[];
  readonly span: DirectiveSpan;
  readonly nameSpan: DirectiveSpan;
  readonly raw: string;
}

/** A registry-validated directive with explicit schema defaults applied. */
export interface ValidatedDirective {
  readonly name: string;
  readonly form: DirectiveForm;
  readonly properties: Readonly<Record<string, DirectiveValue>>;
  readonly span?: import("./source.js").SourceSpan;
}

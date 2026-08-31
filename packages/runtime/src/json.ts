/** A JSON value accepted by the versioned MamboSite wire format. */
export type JsonValue =
  | null
  | boolean
  | number
  | string
  | readonly JsonValue[]
  | JsonObject;

/** A JSON object with immutable values, as produced by generated modules. */
export interface JsonObject {
  readonly [key: string]: JsonValue;
}

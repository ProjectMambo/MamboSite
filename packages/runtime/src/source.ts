/** A one-based location in a UTF-8 source file. */
export interface SourcePosition {
  readonly line: number;
  readonly column: number;
}

/** A half-open range: `start` is inclusive and `end` is exclusive. */
export interface SourceSpan {
  readonly start: SourcePosition;
  readonly end: SourcePosition;
  /** Zero-based UTF-8 byte offset, when retained in this output mode. */
  readonly startByte?: number;
  /** Exclusive zero-based UTF-8 byte offset. */
  readonly endByte?: number;
}

/** A content-root-relative path and range. */
export interface SourceLocation {
  readonly path: string;
  readonly span: SourceSpan;
}

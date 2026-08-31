import type { ParsedDirective } from "./directive.js";
import type { SourceSpan } from "./source.js";

export type ListKind = "bullet" | "ordered";
export type ListDelimiter = "period" | "parenthesis";
export type TableAlignment = "none" | "left" | "center" | "right";
export type AlertKind = "note" | "tip" | "important" | "warning" | "caution";

interface NodeWithChildren {
  readonly span?: SourceSpan;
  readonly children?: readonly MarkdownNode[];
  readonly blockId?: string;
}

export interface DocumentNode {
  readonly type: "document";
}

export interface FrontMatterNode {
  readonly type: "frontMatter";
  readonly literal: string;
}

export interface BlockQuoteNode {
  readonly type: "blockQuote";
}

export interface ListNode {
  readonly type: "list";
  readonly kind: ListKind;
  readonly start: number;
  readonly delimiter: ListDelimiter;
  readonly tight: boolean;
  readonly isTaskList: boolean;
}

export interface ListItemNode {
  readonly type: "listItem";
}

export interface DescriptionListNode {
  readonly type: "descriptionList";
}

export interface DescriptionItemNode {
  readonly type: "descriptionItem";
  readonly tight: boolean;
}

export interface DescriptionTermNode {
  readonly type: "descriptionTerm";
}

export interface DescriptionDetailsNode {
  readonly type: "descriptionDetails";
}

export interface CodeBlockNode {
  readonly type: "codeBlock";
  readonly literal: string;
  readonly info: string;
  readonly fenced: boolean;
  readonly closed: boolean;
}

export interface HtmlBlockNode {
  readonly type: "htmlBlock";
  readonly literal: string;
  readonly blockType: number;
}

export interface ParagraphNode {
  readonly type: "paragraph";
}

export interface HeadingNode {
  readonly type: "heading";
  readonly level: number;
  readonly setext: boolean;
}

export interface ThematicBreakNode {
  readonly type: "thematicBreak";
}

export interface FootnoteDefinitionNode {
  readonly type: "footnoteDefinition";
  readonly name: string;
  readonly totalReferences: number;
}

export interface TableNode {
  readonly type: "table";
  readonly alignments: readonly TableAlignment[];
}

export interface TableRowNode {
  readonly type: "tableRow";
  readonly header: boolean;
}

export interface TableCellNode {
  readonly type: "tableCell";
}

export interface TextNode {
  readonly type: "text";
  readonly value: string;
}

export interface TaskItemNode {
  readonly type: "taskItem";
  readonly checked: boolean;
  /** A single Unicode scalar when present. */
  readonly marker: string | null;
}

export interface SoftBreakNode {
  readonly type: "softBreak";
}

export interface LineBreakNode {
  readonly type: "lineBreak";
}

export interface InlineCodeNode {
  readonly type: "inlineCode";
  readonly literal: string;
}

export interface HtmlInlineNode {
  readonly type: "htmlInline";
  readonly literal: string;
}

export interface RawNode {
  readonly type: "raw";
  readonly literal: string;
}

export interface EmphasisNode {
  readonly type: "emphasis";
}

export interface StrongNode {
  readonly type: "strong";
}

export interface StrikethroughNode {
  readonly type: "strikethrough";
}

export interface HighlightNode {
  readonly type: "highlight";
}

export interface InsertNode {
  readonly type: "insert";
}

export interface SuperscriptNode {
  readonly type: "superscript";
}

export interface LinkNode {
  readonly type: "link";
  readonly destination: string;
  readonly title: string;
}

export interface ImageNode {
  readonly type: "image";
  readonly source: string;
  readonly title: string;
}

export interface FootnoteReferenceNode {
  readonly type: "footnoteReference";
  readonly name: string;
}

export interface MathNode {
  readonly type: "math";
  readonly literal: string;
  readonly display: boolean;
  readonly dollar: boolean;
}

export interface MultilineBlockQuoteNode {
  readonly type: "multilineBlockQuote";
  readonly fenceLength: number;
}

export interface EscapedNode {
  readonly type: "escaped";
}

export interface WikiLinkNode {
  readonly type: "wikiLink";
  readonly destination: string;
}

export interface ObsidianEmbedNode {
  readonly type: "obsidianEmbed";
  readonly destination: string;
  readonly option: string | null;
}

export interface UnderlineNode {
  readonly type: "underline";
}

export interface SubscriptNode {
  readonly type: "subscript";
}

export interface SpoileredTextNode {
  readonly type: "spoileredText";
}

export interface EscapedTagNode {
  readonly type: "escapedTag";
  readonly tag: string;
}

export interface AlertNode {
  readonly type: "alert";
  readonly kind: AlertKind;
  readonly title: string | null;
}

export interface SubtextNode {
  readonly type: "subtext";
}

export interface BlockDirectiveNode {
  readonly type: "blockDirective";
  readonly info: string;
  readonly fenceLength: number;
}

export interface DirectiveNode {
  readonly type: "directive";
  readonly invocation: ParsedDirective;
  readonly fenceLength?: number;
}

/**
 * Exhaustive parser-level union owned by MamboSite. Semantic compiler passes
 * may add separate node unions later without exposing parser-library types.
 */
export type NodeKind =
  | DocumentNode
  | FrontMatterNode
  | BlockQuoteNode
  | ListNode
  | ListItemNode
  | DescriptionListNode
  | DescriptionItemNode
  | DescriptionTermNode
  | DescriptionDetailsNode
  | CodeBlockNode
  | HtmlBlockNode
  | ParagraphNode
  | HeadingNode
  | ThematicBreakNode
  | FootnoteDefinitionNode
  | TableNode
  | TableRowNode
  | TableCellNode
  | TextNode
  | TaskItemNode
  | SoftBreakNode
  | LineBreakNode
  | InlineCodeNode
  | HtmlInlineNode
  | RawNode
  | EmphasisNode
  | StrongNode
  | StrikethroughNode
  | HighlightNode
  | InsertNode
  | SuperscriptNode
  | LinkNode
  | ImageNode
  | FootnoteReferenceNode
  | MathNode
  | MultilineBlockQuoteNode
  | EscapedNode
  | WikiLinkNode
  | ObsidianEmbedNode
  | UnderlineNode
  | SubscriptNode
  | SpoileredTextNode
  | EscapedTagNode
  | AlertNode
  | SubtextNode
  | BlockDirectiveNode
  | DirectiveNode;

/** A parser node plus its optional source location and child nodes. */
export type MarkdownNode = NodeKind & NodeWithChildren;

import { createElement, type ReactNode } from "react";
import type {
  NodeComponentProps,
  NodeRegistry,
} from "@mambosite/react";

function Passthrough({ children }: { readonly children: ReactNode }) {
  return <>{children}</>;
}

function Hidden() {
  return null;
}

function Document({ children }: NodeComponentProps<"document">) {
  return <>{children}</>;
}

function Paragraph({ node, children, state }: NodeComponentProps<"paragraph">) {
  return <p id={node.blockId ? `${state.idPrefix}${node.blockId}` : undefined}>{children}</p>;
}

function BlockQuote({ node, children, state }: NodeComponentProps<"blockQuote">) {
  return <blockquote id={node.blockId ? `${state.idPrefix}${node.blockId}` : undefined}>{children}</blockquote>;
}

function MultilineBlockQuote({ node, children, state }: NodeComponentProps<"multilineBlockQuote">) {
  return <blockquote id={node.blockId ? `${state.idPrefix}${node.blockId}` : undefined}>{children}</blockquote>;
}

function List({ node, children }: NodeComponentProps<"list">) {
  return node.kind === "ordered"
    ? <ol start={node.start}>{children}</ol>
    : <ul>{children}</ul>;
}

function ListItem({ children }: NodeComponentProps<"listItem">) {
  return <li>{children}</li>;
}

function DescriptionList({ children }: NodeComponentProps<"descriptionList">) {
  return <dl>{children}</dl>;
}

function DescriptionTerm({ children }: NodeComponentProps<"descriptionTerm">) {
  return <dt>{children}</dt>;
}

function DescriptionDetails({ children }: NodeComponentProps<"descriptionDetails">) {
  return <dd>{children}</dd>;
}

function CodeBlock({ node, state }: NodeComponentProps<"codeBlock">) {
  const language = node.info.split(/\s+/, 1)[0] || undefined;
  return (
    <pre id={node.blockId ? `${state.idPrefix}${node.blockId}` : undefined}>
      <code data-language={language}>{node.literal}</code>
    </pre>
  );
}

function HtmlBlock({ node }: NodeComponentProps<"htmlBlock">) {
  return <code className="mambo-raw-html">{node.literal}</code>;
}

function HtmlInline({ node }: NodeComponentProps<"htmlInline">) {
  return <code className="mambo-raw-html">{node.literal}</code>;
}

function Heading({ children, model }: NodeComponentProps<"heading">) {
  return createElement(`h${model.level}`, { id: model.id }, children);
}

function ThematicBreak() {
  return <hr />;
}

function FootnoteDefinition({ node, children, state }: NodeComponentProps<"footnoteDefinition">) {
  return <aside className="mambo-footnote" id={`${state.idPrefix}footnote-${node.name}`}>{children}</aside>;
}

function Table({ children }: NodeComponentProps<"table">) {
  return <div className="mambo-table-scroll"><table><tbody>{children}</tbody></table></div>;
}

function TableRow({ children }: NodeComponentProps<"tableRow">) {
  return <tr>{children}</tr>;
}

function TableCell({ children, model }: NodeComponentProps<"tableCell">) {
  return model.header ? <th>{children}</th> : <td>{children}</td>;
}

function Text({ node }: NodeComponentProps<"text">) {
  return node.value;
}

function TaskItem({ node }: NodeComponentProps<"taskItem">) {
  return <input aria-label="Task status" checked={node.checked} readOnly type="checkbox" />;
}

function SoftBreak() {
  return "\n";
}

function LineBreak() {
  return <br />;
}

function InlineCode({ node }: NodeComponentProps<"inlineCode">) {
  return <code>{node.literal}</code>;
}

function Raw({ node }: NodeComponentProps<"raw">) {
  return node.literal;
}

function Emphasis({ children }: NodeComponentProps<"emphasis">) {
  return <em>{children}</em>;
}

function Strong({ children }: NodeComponentProps<"strong">) {
  return <strong>{children}</strong>;
}

function Strikethrough({ children }: NodeComponentProps<"strikethrough">) {
  return <del>{children}</del>;
}

function Highlight({ children }: NodeComponentProps<"highlight">) {
  return <mark>{children}</mark>;
}

function Insert({ children }: NodeComponentProps<"insert">) {
  return <ins>{children}</ins>;
}

function Superscript({ children }: NodeComponentProps<"superscript">) {
  return <sup>{children}</sup>;
}

function Underline({ children }: NodeComponentProps<"underline">) {
  return <u>{children}</u>;
}

function Subscript({ children }: NodeComponentProps<"subscript">) {
  return <sub>{children}</sub>;
}

function Spoiler({ children }: NodeComponentProps<"spoileredText">) {
  return <span className="mambo-spoiler" tabIndex={0}>{children}</span>;
}

function EscapedTag({ node }: NodeComponentProps<"escapedTag">) {
  return `<${node.tag}>`;
}

function LinkNode({ children, model, runtime }: NodeComponentProps<"link">) {
  const Link = runtime.registry.primitives.Link;
  return <Link href={model.href}>{children}</Link>;
}

function WikiLink({ children, model, runtime }: NodeComponentProps<"wikiLink">) {
  const Link = runtime.registry.primitives.Link;
  return <Link href={model.href}>{children || model.label}</Link>;
}

function ImageNode({ node, model, runtime }: NodeComponentProps<"image">) {
  const Image = runtime.registry.primitives.Image;
  return (
    <Image
      alt={model.alt}
      src={node.source}
      {...(node.title ? { title: node.title } : {})}
    />
  );
}

function FootnoteReference({ node, state }: NodeComponentProps<"footnoteReference">) {
  return <sup><a href={`#${state.idPrefix}footnote-${node.name}`}>[{node.name}]</a></sup>;
}

function Math({ node }: NodeComponentProps<"math">) {
  return node.display
    ? <pre className="mambo-math"><code>{node.literal}</code></pre>
    : <code className="mambo-math">{node.literal}</code>;
}

function Embed({ node, model, runtime }: NodeComponentProps<"obsidianEmbed">) {
  if (model.error === "recursive") {
    return <p className="mambo-message mambo-message--error">Recursive embed blocked.</p>;
  }
  if (model.error === "fragment") {
    const Link = runtime.registry.primitives.Link;
    return (
      <p className="mambo-unsupported">
        Fragment embeds are preserved but not rendered by this theme version.
        {model.sourceHref ? <> <Link href={model.sourceHref}>Open source fragment</Link></> : null}
      </p>
    );
  }
  if (model.kind === "page") {
    return (
      <section
        className="mambo-embedded-page"
        data-source={model.target?.sourcePath}
        id={model.instanceId}
      >
        {model.content}
      </section>
    );
  }
  const Image = runtime.registry.primitives.Image;
  return <Image alt={node.option ?? ""} src={model.source} />;
}

function Alert({ node, children }: NodeComponentProps<"alert">) {
  return (
    <aside className={`mambo-callout mambo-callout--${node.kind}`}>
      {node.title ? <strong>{node.title}</strong> : null}
      {children}
    </aside>
  );
}

function Subtext({ children }: NodeComponentProps<"subtext">) {
  return <small>{children}</small>;
}

function UnsupportedBlockDirective({ node }: NodeComponentProps<"blockDirective">) {
  return <div className="mambo-unsupported">Unsupported directive: {node.info}</div>;
}

export const defaultNodeRegistry = Object.freeze({
  document: Document,
  frontMatter: Hidden,
  blockQuote: BlockQuote,
  list: List,
  listItem: ListItem,
  descriptionList: DescriptionList,
  descriptionItem: Passthrough,
  descriptionTerm: DescriptionTerm,
  descriptionDetails: DescriptionDetails,
  codeBlock: CodeBlock,
  htmlBlock: HtmlBlock,
  paragraph: Paragraph,
  heading: Heading,
  thematicBreak: ThematicBreak,
  footnoteDefinition: FootnoteDefinition,
  table: Table,
  tableRow: TableRow,
  tableCell: TableCell,
  text: Text,
  taskItem: TaskItem,
  softBreak: SoftBreak,
  lineBreak: LineBreak,
  inlineCode: InlineCode,
  htmlInline: HtmlInline,
  raw: Raw,
  emphasis: Emphasis,
  strong: Strong,
  strikethrough: Strikethrough,
  highlight: Highlight,
  insert: Insert,
  superscript: Superscript,
  link: LinkNode,
  image: ImageNode,
  footnoteReference: FootnoteReference,
  math: Math,
  multilineBlockQuote: MultilineBlockQuote,
  escaped: Passthrough,
  wikiLink: WikiLink,
  obsidianEmbed: Embed,
  underline: Underline,
  subscript: Subscript,
  spoileredText: Spoiler,
  escapedTag: EscapedTag,
  alert: Alert,
  subtext: Subtext,
  blockDirective: UnsupportedBlockDirective,
}) satisfies NodeRegistry;

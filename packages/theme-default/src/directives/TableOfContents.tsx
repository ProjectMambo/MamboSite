import type { DirectiveComponentProps } from "@mambosite/react";
import { TableOfContentsBehavior } from "./TableOfContentsBehavior.js";

export function TableOfContents({
  config,
  model,
}: DirectiveComponentProps<"toc">) {
  if (model.headings.length === 0) return null;
  const List = config.ordered ? "ol" : "ul";
  const list = () => (
    <List>
      {model.headings.map((heading) => (
        <li key={`${heading.id}-${heading.span?.startByte ?? 0}`} data-level={heading.level}>
          <a href={`#${heading.id}`} data-mambo-toc-target={heading.id}>{heading.text}</a>
        </li>
      ))}
    </List>
  );
  return (
    <nav className="mambo-toc" aria-label={config.title} data-collapse={config.collapse}>
      <TableOfContentsBehavior headingIds={model.headings.map((heading) => heading.id)} />
      <div className="mambo-toc__expanded">
        <p className="mambo-toc__title">{config.title}</p>
        {list()}
      </div>
      <details className="mambo-toc__disclosure">
        <summary>{config.title}</summary>
        {list()}
      </details>
    </nav>
  );
}

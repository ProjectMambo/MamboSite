import type { DirectiveComponentProps } from "@mambosite/react";

export function TableOfContents({
  config,
  model,
}: DirectiveComponentProps<"toc">) {
  if (model.headings.length === 0) return null;
  const List = config.ordered ? "ol" : "ul";
  const body = (
    <List>
      {model.headings.map((heading) => (
        <li key={`${heading.id}-${heading.span?.startByte ?? 0}`} data-level={heading.level}>
          <a href={`#${heading.id}`}>{heading.text}</a>
        </li>
      ))}
    </List>
  );
  return (
    <nav className="mambo-toc" aria-label={config.title}>
      {config.collapse ? (
        <details>
          <summary>{config.title}</summary>
          {body}
        </details>
      ) : (
        <>
          <p className="mambo-toc__title">{config.title}</p>
          {body}
        </>
      )}
    </nav>
  );
}

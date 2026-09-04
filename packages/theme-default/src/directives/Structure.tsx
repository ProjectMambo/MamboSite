import type { CSSProperties } from "react";
import type { DirectiveComponentProps } from "@mambosite/react";
import { CollectionView } from "../CollectionView.js";

export function HiddenDirective() {
  return null;
}

export function Breadcrumbs({
  config,
  model,
  runtime,
}: DirectiveComponentProps<"breadcrumbs">) {
  const Link = runtime.registry.primitives.Link;
  return (
    <nav className="mambo-breadcrumbs" aria-label="Breadcrumbs">
      {model.items.map((item, index) => (
        <span key={item.href}>
          {index > 0 ? <span aria-hidden="true">{config.separator}</span> : null}
          <Link current={item.current} href={item.href}>{item.label}</Link>
        </span>
      ))}
    </nav>
  );
}

export function ChildrenCollection({
  config,
  model,
  runtime,
}: DirectiveComponentProps<"children">) {
  if (config.depth !== 1) {
    return <p className="mambo-unsupported">Nested child depth is not available in this runtime version.</p>;
  }
  if (config.view === "tree" || config.view === "table") {
    return <p className="mambo-unsupported">The {config.view} child view is not available in this runtime version.</p>;
  }
  return (
    <CollectionView
      columns={config.columns}
      empty={model.emptyMessage}
      items={model.items}
      runtime={runtime}
      show={config.show}
      view={config.view}
    />
  );
}

export function RelatedCollection({
  config,
  model,
  runtime,
}: DirectiveComponentProps<"related">) {
  return (
    <CollectionView
      empty={model.emptyMessage}
      items={model.items}
      runtime={runtime}
      show={config.show}
      view={config.view}
    />
  );
}

export function BacklinkCollection({
  config,
  model,
  runtime,
}: DirectiveComponentProps<"backlinks">) {
  return (
    <CollectionView
      empty={model.emptyMessage}
      items={model.items}
      runtime={runtime}
      show={config.show}
      view={config.view}
    />
  );
}

export function Gallery({
  config,
  model,
  runtime,
}: DirectiveComponentProps<"gallery">) {
  if (config.view === "masonry" || config.view === "carousel") {
    return <p className="mambo-unsupported">The {config.view} gallery is not available in this runtime version.</p>;
  }
  const show = [
    "cover",
    "title",
    ...(config.captions ? ["description"] : []),
  ];
  return (
    <CollectionView
      columns={config.columns}
      empty={model.emptyMessage}
      fit={config.fit}
      items={model.items}
      runtime={runtime}
      show={show}
      view={config.view === "grid" ? "gallery" : config.view}
    />
  );
}

export function Include({
  config,
  model,
  runtime,
}: DirectiveComponentProps<"include">) {
  if (model.error === "missing") {
    return <p className="mambo-message mambo-message--error">Included page could not be resolved.</p>;
  }
  if (model.error === "recursive") {
    return <p className="mambo-message mambo-message--error">Recursive include blocked.</p>;
  }
  if (model.error === "fragment") {
    return <p className="mambo-unsupported">Fragment includes are not available in this runtime version.</p>;
  }
  const Link = runtime.registry.primitives.Link;
  const content = (
    <>
      {config.showTitle && model.target ? <h2>{model.target.title}</h2> : null}
      {model.content}
      {config.showSource && model.target && model.sourceHref ? (
        <p><Link href={model.sourceHref}>Read {model.target.title}</Link></p>
      ) : null}
    </>
  );
  if (config.mode === "inline") return content;
  return (
    <section className="mambo-embedded-page" data-source={model.target?.sourcePath} id={model.instanceId}>
      {content}
    </section>
  );
}

export function Button({
  config,
  model,
  runtime,
}: DirectiveComponentProps<"button">) {
  const Link = runtime.registry.primitives.Link;
  if (model.error === "unsafe") {
    return <p className="mambo-message mambo-message--error">Unsafe button link blocked.</p>;
  }
  return (
    <p className="mambo-button-row">
      <Link
        className={`mambo-button mambo-button--${config.variant}`}
        href={model.href}
        newTab={config.external && !model.href.startsWith("mailto:")}
      >
        {config.icon ? <span aria-hidden="true">{config.icon}</span> : null}
        {config.label}
      </Link>
    </p>
  );
}

export function Section({
  config,
  children,
  state,
}: DirectiveComponentProps<"section">) {
  return (
    <section
      className={`mambo-section mambo-section--${config.tone} mambo-section--${config.width}`}
      data-align={config.align}
      id={config.id ? `${state.idPrefix}${config.id}` : undefined}
    >
      {children}
    </section>
  );
}

export function Columns({
  config,
  children,
}: DirectiveComponentProps<"columns">) {
  const style = { "--mambo-column-count": config.count } as CSSProperties;
  const collapse = {
    sm: "compact",
    md: "content",
    lg: "wide",
    never: "never",
  }[config.collapseAt];
  return (
    <div
      className="mambo-columns"
      data-collapse={collapse}
      data-columns={config.count}
      data-gap={config.gap}
      data-mambo-columns
      style={style}
    >
      {children}
    </div>
  );
}

export function Column({ children }: DirectiveComponentProps<"column">) {
  return <div className="mambo-column" data-mambo-accent-item>{children}</div>;
}

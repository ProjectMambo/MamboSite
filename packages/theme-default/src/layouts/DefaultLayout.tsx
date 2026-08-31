import type { PageLayoutProps } from "@mambosite/react";

export function DefaultLayout({
  page,
  config,
  children,
  sidebar,
  showGeneratedTitle,
  parentHref,
  runtime,
}: PageLayoutProps) {
  const Link = runtime.registry.primitives.Link;
  return (
    <div
      className={`mambo-page-frame mambo-page-frame--${config.width}`}
      data-layout={config.layout}
      data-mambo-page-frame
      data-sidebar={Boolean(sidebar)}
    >
      <article className="mambo-page-article">
        {showGeneratedTitle ? <h1>{page.title}</h1> : null}
        {children}
        {parentHref ? (
          <p className="mambo-back-link"><Link href={parentHref}>← Back</Link></p>
        ) : null}
      </article>
      {sidebar ? <aside className="mambo-page-sidebar">{sidebar}</aside> : null}
    </div>
  );
}

export function ArticleLayout(props: PageLayoutProps) {
  const width = props.config.width === "normal" ? "narrow" : props.config.width;
  return <DefaultLayout {...props} config={{ ...props.config, layout: "article", width }} />;
}

export function DocsLayout(props: PageLayoutProps) {
  return <DefaultLayout {...props} config={{ ...props.config, layout: "docs", sidebar: true }} />;
}

export function ProjectLayout(props: PageLayoutProps) {
  return <DefaultLayout {...props} config={{ ...props.config, layout: "project" }} />;
}

export function CollectionLayout(props: PageLayoutProps) {
  const width = props.config.width === "normal" ? "wide" : props.config.width;
  return <DefaultLayout {...props} config={{ ...props.config, layout: "collection", width }} />;
}

export function HomeLayout(props: PageLayoutProps) {
  const width = props.config.width === "normal" ? "wide" : props.config.width;
  return <DefaultLayout {...props} config={{ ...props.config, layout: "home", width }} />;
}

export function GalleryLayout(props: PageLayoutProps) {
  const width = props.config.width === "normal" ? "wide" : props.config.width;
  return <DefaultLayout {...props} config={{ ...props.config, layout: "gallery", width }} />;
}

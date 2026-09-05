import { objectValue, type JsonObject, type JsonValue } from "@mambosite/runtime";
import { MarkdownRenderer, type SiteFooterProps } from "@mambosite/react";

interface FooterLink {
  readonly label: string;
  readonly href: string;
}

export function SiteFooter({ runtime }: SiteFooterProps) {
  const data = objectValue(runtime.store.entryPage.data.footer);
  const copyright = typeof data?.copyright === "string"
    ? data.copyright
    : runtime.store.manifest.site.title;
  const links = footerLinks(data?.links);
  const Link = runtime.registry.primitives.Link;
  const entry = runtime.store.entryPage;
  const authored = entry.body.children?.find(
    (node) => node.type === "directive" && node.invocation.name === "footer",
  );

  return (
    <footer className="mambo-site-footer" data-mambo-footer>
      <p>© {copyright}</p>
      {authored ? (
        <MarkdownRenderer nodes={authored.children ?? []} page={entry} runtime={runtime} />
      ) : null}
      <nav aria-label="Footer navigation">
        {links.map((item) => (
          <Link href={item.href} key={item.href}>{item.label}</Link>
        ))}
      </nav>
    </footer>
  );
}

function footerLinks(value: JsonValue | undefined): readonly FooterLink[] {
  if (!Array.isArray(value)) return [];
  return value.flatMap((item) => {
    if (!item || typeof item !== "object" || Array.isArray(item)) return [];
    const object = item as JsonObject;
    return typeof object.label === "string" && typeof object.href === "string"
      ? [{ label: object.label, href: object.href }]
      : [];
  });
}

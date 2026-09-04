import { type JsonObject, type JsonValue } from "@mambosite/runtime";
import type { SiteHeaderProps } from "@mambosite/react";
import {
  HeaderBehavior,
  NavigationMenu,
  SiteClock,
  ThemeButton,
} from "./HeaderClient.js";

interface NavigationItem {
  readonly label: string;
  readonly href: string;
}

export function SiteHeader({ runtime }: SiteHeaderProps) {
  const items = navigationItems(runtime.store.entryPage.data.navigation);
  const [brand, ...navigation] = items;
  const theme = runtime.options.theme ?? {
    defaultScheme: "dark",
    schemes: ["dark", "light"],
  };
  const Link = runtime.registry.primitives.Link;
  const headerId = "mambo-site-header";
  return (
    <header className="mambo-site-header" id={headerId}>
      <HeaderBehavior targetId={headerId} />
      <div className="mambo-site-header__inner" data-mambo-header-inner>
        <Link className="mambo-site-brand" href="/">
          {brand?.label ?? runtime.store.manifest.site.title}
        </Link>
        <NavigationMenu>
          {navigation.map((item) => (
            <Link href={item.href} key={`${item.href}-${item.label}`}>{item.label}</Link>
          ))}
        </NavigationMenu>
        <div className="mambo-site-tools">
          <ThemeButton defaultScheme={theme.defaultScheme} schemes={theme.schemes} />
          <SiteClock locale={runtime.options.locale ?? runtime.store.manifest.site.language} />
        </div>
      </div>
    </header>
  );
}

function navigationItems(value: JsonValue | undefined): readonly NavigationItem[] {
  if (!Array.isArray(value)) return [];
  return value.flatMap((item) => {
    if (!item || typeof item !== "object" || Array.isArray(item)) return [];
    const object = item as JsonObject;
    return typeof object.label === "string" && typeof object.href === "string"
      ? [{ label: object.label, href: object.href }]
      : [];
  });
}

import Link from "next/link.js";
import type { LinkPrimitiveProps } from "@mambosite/react";

export function NextLink({
  href,
  children,
  accentItem = false,
  className,
  title,
  newTab = false,
  current = false,
}: LinkPrimitiveProps) {
  return (
    <Link
      aria-current={current ? "page" : undefined}
      className={className}
      data-mambo-accent-item={accentItem || undefined}
      href={href}
      rel={newTab ? "noreferrer" : undefined}
      target={newTab ? "_blank" : undefined}
      title={title}
    >
      {children}
    </Link>
  );
}

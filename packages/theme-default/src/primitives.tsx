import type {
  ImagePrimitiveProps,
  LinkPrimitiveProps,
  PrimitiveRegistry,
} from "@mambosite/react";

export function Link({
  href,
  children,
  accentItem = false,
  className,
  title,
  newTab = false,
  current = false,
}: LinkPrimitiveProps) {
  return (
    <a
      aria-current={current ? "page" : undefined}
      className={className}
      data-mambo-accent-item={accentItem || undefined}
      href={href}
      rel={newTab ? "noreferrer" : undefined}
      target={newTab ? "_blank" : undefined}
      title={title}
    >
      {children}
    </a>
  );
}

export function Image({
  src,
  alt,
  className,
  title,
  decorative = false,
}: ImagePrimitiveProps) {
  // Compiler assets are static, but authored media has no mandatory dimensions.
  // The framework adapter may replace this primitive with a compatible image.
  return (
    <img
      alt={decorative ? "" : alt}
      aria-hidden={decorative || undefined}
      className={className}
      src={src}
      title={title}
    />
  );
}

export const defaultPrimitives = Object.freeze({ Link, Image }) satisfies PrimitiveRegistry;

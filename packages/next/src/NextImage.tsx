import type { ImagePrimitiveProps } from "@mambosite/react";

export interface NextImageProps extends ImagePrimitiveProps {
  readonly basePath?: string;
}

/** Static-export-safe image primitive with exact-once base-path handling. */
export function NextImage({
  basePath = "",
  src,
  alt,
  className,
  title,
  decorative = false,
}: NextImageProps) {
  return (
    <img
      alt={decorative ? "" : alt}
      aria-hidden={decorative || undefined}
      className={className}
      src={prefixBasePath(src, basePath)}
      title={title}
    />
  );
}

export function prefixBasePath(source: string, basePath: string): string {
  if (!source.startsWith("/") || source.startsWith("//")) return source;
  const base = normalizeBasePath(basePath);
  if (!base || source === base || source.startsWith(`${base}/`)) return source;
  return source === "/" ? `${base}/` : `${base}${source}`;
}

function normalizeBasePath(basePath: string): string {
  const segments = basePath.split("/").filter(Boolean);
  return segments.length === 0 ? "" : `/${segments.join("/")}`;
}

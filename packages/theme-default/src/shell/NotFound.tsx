import type { NotFoundProps } from "@mambosite/react";

export function NotFound({ runtime }: NotFoundProps) {
  const Link = runtime.registry.primitives.Link;
  return (
    <article className="mambo-not-found">
      <p className="mambo-eyebrow">404</p>
      <h1>Page not found</h1>
      <p>The generated content graph does not contain this route.</p>
      <p><Link className="mambo-button" href="/">Return home</Link></p>
    </article>
  );
}

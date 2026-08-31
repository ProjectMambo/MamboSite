import type {
  PageLayoutProps,
  RegistryFallbacks,
  UnsupportedDirectiveProps,
  UnsupportedNodeProps,
} from "@mambosite/react";

function UnsupportedNode({ node, children }: UnsupportedNodeProps) {
  return (
    <div className="mambo-unsupported" data-node={node.type}>
      {children || `Unsupported node: ${node.type}`}
    </div>
  );
}

function UnsupportedDirective({ name, children }: UnsupportedDirectiveProps) {
  return (
    <div className="mambo-unsupported" data-directive={name}>
      Unsupported component: {name}
      {children}
    </div>
  );
}

function FallbackLayout({ children }: PageLayoutProps) {
  return <article className="mambo-page-article">{children}</article>;
}

export const defaultFallbacks = Object.freeze({
  Node: UnsupportedNode,
  Directive: UnsupportedDirective,
  Layout: FallbackLayout,
}) satisfies RegistryFallbacks;

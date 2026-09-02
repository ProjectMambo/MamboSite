import type { ReactNode } from "react";
import {
  prefixBasePath,
  siteMetadata,
  themeBootstrapScript,
} from "@mambosite/next";
import { MamboSiteFrame } from "@mambosite/react";
import "./globals.css";
import { runtime, theme, themeStylesheetHref } from "../mambo/runtime";

export const metadata = siteMetadata(runtime);

const themeHref = prefixBasePath(
  themeStylesheetHref,
  runtime.store.manifest.site.basePath,
);

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html
      data-theme={theme.defaultScheme}
      lang={runtime.store.manifest.site.language}
      suppressHydrationWarning
    >
      <head>
        <link href={themeHref} rel="stylesheet" />
        <script
          dangerouslySetInnerHTML={{
            __html: themeBootstrapScript(theme.defaultScheme),
          }}
        />
      </head>
      <body>
        <MamboSiteFrame runtime={runtime}>{children}</MamboSiteFrame>
      </body>
    </html>
  );
}

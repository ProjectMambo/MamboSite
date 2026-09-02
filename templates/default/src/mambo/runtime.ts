import { createNextRuntime } from "@mambosite/next";
import { defaultRegistry } from "@mambosite/theme-default";
import manifest from "../generated/mambo/manifest";
import { pages } from "../generated/mambo/pages";
import {
  theme,
  themeStylesheetHref,
} from "../generated/mambo/theme";

export const runtime = createNextRuntime({
  manifest,
  pages,
  registry: defaultRegistry,
  options: {
    locale: manifest.site.language,
    theme: {
      defaultScheme: theme.defaultScheme,
      schemes: theme.schemes,
    },
  },
});

export { theme, themeStylesheetHref };

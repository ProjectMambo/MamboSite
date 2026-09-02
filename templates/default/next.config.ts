import type { NextConfig } from "next";
import manifest from "./src/generated/mambo/manifest";

const config: NextConfig = {
  output: "export",
  basePath: manifest.site.basePath,
  trailingSlash: manifest.site.trailingSlash,
  images: { unoptimized: true },
};

export default config;

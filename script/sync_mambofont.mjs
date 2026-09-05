#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const provider = process.env.MAMBOFONT_BIN ?? "mbfont";
const version = "0.2.4";
const fontDirectory = join(root, "packages/theme-default/src/fonts");
const stylesheet = join(root, "packages/theme-default/src/styles/mambofont.css");
const check = process.argv.slice(2).includes("--check");
const faces = [
  ["Regular", 400],
  ["Medium", 500],
  ["SemiBold", 600],
  ["Bold", 700],
].map(([style, weight]) => ({
  filename: `MamboFont-${style}_v${version}.woff2`,
  style,
  weight,
}));

if (process.argv.slice(2).some((argument) => argument !== "--check")) {
  throw new Error("usage: node script/sync_mambofont.mjs [--check]");
}

const temporary = mkdtempSync(join(tmpdir(), "mambosite-font-"));

try {
  execFileSync(provider, ["compile", version, "--format", "woff2", "--out", temporary], {
    cwd: root,
    env: { ...process.env, SOURCE_DATE_EPOCH: "0" },
    stdio: "ignore",
  });
  const css = renderStylesheet();

  if (check) {
    for (const { filename } of faces) {
      const actual = readFileSync(join(fontDirectory, filename));
      const expected = readFileSync(join(temporary, filename));
      if (!actual.equals(expected)) throw new Error(`generated font is stale: ${filename}`);
    }
    if (readFileSync(stylesheet, "utf8") !== css) {
      throw new Error(`generated stylesheet is stale: ${basename(stylesheet)}`);
    }
    console.log("MamboFont assets are current");
  } else {
    mkdirSync(fontDirectory, { recursive: true });
    for (const { filename } of faces) {
      copyFileSync(join(temporary, filename), join(fontDirectory, filename));
    }
    writeFileSync(stylesheet, css);
    console.log(`updated ${fontDirectory}`);
  }
} finally {
  rmSync(temporary, { force: true, recursive: true });
}

function renderStylesheet() {
  return [
    `/* Generated from MamboFont v${version} by \`node script/sync_mambofont.mjs\`; do not edit. */`,
    "",
    ...faces.flatMap(({ filename, weight }) => [
      "@font-face {",
      '  font-family: "MamboFont";',
      `  src: url("../fonts/${filename}") format("woff2");`,
      `  font-weight: ${weight};`,
      "  font-style: normal;",
      "  font-display: swap;",
      "}",
      "",
    ]),
  ].join("\n");
}

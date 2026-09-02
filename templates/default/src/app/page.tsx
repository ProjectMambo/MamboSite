import { metadataForPage } from "@mambosite/next";
import { MamboPage } from "@mambosite/react";
import { runtime } from "../mambo/runtime";

const page = runtime.store.entryPage;

export const metadata = metadataForPage(page);

export default function HomePage() {
  return <MamboPage page={page} runtime={runtime} />;
}

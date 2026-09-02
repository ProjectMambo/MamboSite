import {
  metadataForPage,
  pageFromSegments,
  staticPageParams,
} from "@mambosite/next";
import { MamboPage } from "@mambosite/react";
import { notFound } from "next/navigation";
import { runtime } from "../../mambo/runtime";

interface PageProps {
  readonly params: Promise<{ readonly slug: string[] }>;
}

export const dynamicParams = false;

export function generateStaticParams() {
  return [...staticPageParams(runtime)];
}

export async function generateMetadata({ params }: PageProps) {
  const { slug } = await params;
  return metadataForPage(pageFromSegments(runtime, slug));
}

export default async function ContentPage({ params }: PageProps) {
  const { slug } = await params;
  const page = pageFromSegments(runtime, slug);
  if (!page) notFound();

  return <MamboPage page={page} runtime={runtime} />;
}

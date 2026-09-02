import { MamboNotFound } from "@mambosite/react";
import { runtime } from "../mambo/runtime";

export default function NotFound() {
  return <MamboNotFound runtime={runtime} />;
}

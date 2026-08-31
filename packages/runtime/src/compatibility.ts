export const SUPPORTED_SCHEMA_VERSIONS = Object.freeze([1] as const);

export function assertCompatibleSchema(
  schemaVersion: number,
  source = "generated MamboSite content",
): void {
  if (!(SUPPORTED_SCHEMA_VERSIONS as readonly number[]).includes(schemaVersion)) {
    throw new Error(
      `${source} uses schema ${schemaVersion}; this runtime supports ${SUPPORTED_SCHEMA_VERSIONS.join(", ")}`,
    );
  }
}

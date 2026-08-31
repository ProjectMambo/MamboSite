import {
  formatDate,
  pageValue,
  stripWebTag,
  type JsonValue,
} from "@mambosite/runtime";
import type { DirectiveComponentProps, MamboRuntime } from "@mambosite/react";

const LABELS: Readonly<Record<string, string>> = {
  date: "Published",
  updated: "Updated",
  period: "Period",
  description: "Description",
  tags: "Tags",
  wikiUrl: "Website",
  githubUrl: "GitHub",
};

export function Metadata({ page, config, runtime }: DirectiveComponentProps<"meta">) {
  const entries = config.show.flatMap((field) => {
    const value = pageValue(page, field);
    if (value == null || value === "" || (Array.isArray(value) && value.length === 0)) {
      return config.empty === "placeholder" ? [[field, "—"] as const] : [];
    }
    return [[field, value] as const];
  });
  if (entries.length === 0) return null;

  return (
    <dl className={`mambo-metadata mambo-metadata--${config.style}`} data-mambo-metadata>
      {entries.map(([field, value]) => (
        <div className="mambo-metadata__item" key={field}>
          <dt>{LABELS[field] ?? field}</dt>
          <dd>{renderValue(field, value, runtime)}</dd>
        </div>
      ))}
    </dl>
  );
}

function renderValue(field: string, value: JsonValue, runtime: MamboRuntime) {
  if (field === "tags" && Array.isArray(value)) {
    return (
      <span className="mambo-tag-list">
        {value.map((tag) => typeof tag === "string" ? (
          <span className="mambo-tag" key={tag}>{stripWebTag(tag)}</span>
        ) : null)}
      </span>
    );
  }
  if ((field === "date" || field === "updated") && typeof value === "string") {
    return <time dateTime={value}>{formatDate(value, runtime.options.locale)}</time>;
  }
  if (typeof value === "string" && /Url$/.test(field)) {
    const Link = runtime.registry.primitives.Link;
    return <Link href={value}>{value}</Link>;
  }
  if (["string", "number", "boolean"].includes(typeof value)) return String(value);
  return <code>{JSON.stringify(value)}</code>;
}

import { formatDate, stripWebTag, type PageRecord } from "@mambosite/runtime";
import type { MamboRuntime } from "@mambosite/react";

export interface CollectionViewProps {
  readonly items: readonly PageRecord[];
  readonly runtime: MamboRuntime;
  readonly view?: string;
  readonly columns?: number;
  readonly show?: readonly string[];
  readonly empty?: string;
  readonly fit?: "cover" | "contain" | "natural";
}

export function CollectionView({
  items,
  runtime,
  view = "list",
  columns = 3,
  show = [],
  empty = "Nothing here yet.",
  fit = "cover",
}: CollectionViewProps) {
  if (items.length === 0) {
    return empty ? <p className="mambo-empty-state">{empty}</p> : null;
  }
  if (view === "hidden") return null;

  const visible = new Set(show);
  const gallery = view === "gallery" || view === "masonry" || view === "carousel";
  const grid = gallery || view === "grid" || view === "cards";
  const Link = runtime.registry.primitives.Link;
  const Image = runtime.registry.primitives.Image;
  const requestedColumns = grid ? Math.min(6, Math.max(1, Math.trunc(columns))) : 1;

  return (
    <div
      className={`mambo-collection mambo-collection--${grid ? "grid" : view}`}
      data-columns={requestedColumns}
      data-fit={fit}
      data-mambo-collection
      data-view={view}
    >
      {items.map((item) => (
        <Link
          accentItem
          className="mambo-content-card"
          href={item.route}
          key={item.id}
        >
          {(gallery || visible.has("cover")) && item.cover ? (
            <Image
              alt=""
              className="mambo-content-card__image"
              decorative
              src={item.cover}
            />
          ) : null}
          <div className="mambo-content-card__body">
            {item.date && (visible.has("date") || view === "list") ? (
              <time className="mambo-content-card__date" dateTime={item.date}>
                {formatDate(item.date, runtime.options.locale)}
              </time>
            ) : null}
            <h3>{item.title}</h3>
            {item.description && (show.length === 0 || visible.has("description")) ? (
              <p>{item.description}</p>
            ) : null}
            {item.tags.length > 0 && visible.has("tags") ? (
              <div className="mambo-tag-list">
                {item.tags.map((tag) => (
                  <span className="mambo-tag" key={tag}>{stripWebTag(tag)}</span>
                ))}
              </div>
            ) : null}
          </div>
        </Link>
      ))}
    </div>
  );
}

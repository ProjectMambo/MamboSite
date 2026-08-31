import { objectValue, type JsonObject } from "@mambosite/runtime";
import type { DirectiveComponentProps } from "@mambosite/react";

function stringField(object: JsonObject | undefined, key: string): string | undefined {
  const value = object?.[key];
  return typeof value === "string" ? value : undefined;
}

export function Hero({ page, config, runtime }: DirectiveComponentProps<"hero">) {
  const hero = objectValue(page.data.hero);
  const image = config.image ?? page.cover;
  const quote = stringField(hero, "quote");
  const attribution = stringField(hero, "attribution");
  const Image = runtime.registry.primitives.Image;

  return (
    <header
      className={`mambo-hero mambo-hero--${config.align}`}
      data-align={config.align}
      data-mambo-hero
    >
      {image ? (
        <Image alt="" className="mambo-hero__image" decorative src={image} />
      ) : null}
      <div className="mambo-hero__copy">
        {config.showTitle ? <h1>{page.title}</h1> : null}
        {config.showDescription && page.description ? <p>{page.description}</p> : null}
        {quote ? (
          <blockquote className="mambo-hero__quote">
            <p>“{quote}”</p>
            {attribution ? <cite>— {attribution}</cite> : null}
          </blockquote>
        ) : null}
        {config.showMeta && page.tags.length > 0 ? (
          <p className="mambo-hero__meta">{page.tags.join(" · ")}</p>
        ) : null}
      </div>
    </header>
  );
}

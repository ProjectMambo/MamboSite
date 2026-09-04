"use client";

import { useEffect, useRef } from "react";

export function activeHeadingIndex(
  headingTops: readonly number[],
  threshold: number,
  atBottom: boolean,
) {
  if (headingTops.length === 0) return -1;
  if (atBottom) return headingTops.length - 1;

  let active = -1;
  for (let index = 0; index < headingTops.length; index += 1) {
    if ((headingTops[index] ?? Number.POSITIVE_INFINITY) > threshold) break;
    active = index;
  }
  return active;
}

export function TableOfContentsBehavior({
  headingIds,
}: {
  readonly headingIds: readonly string[];
}) {
  const marker = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    const navigation = marker.current?.closest<HTMLElement>(".mambo-toc");
    if (!navigation) return;

    const headings = headingIds
      .map((id) => document.getElementById(id))
      .filter((heading): heading is HTMLElement => heading !== null);
    const links = Array.from(
      navigation.querySelectorAll<HTMLAnchorElement>("a[data-mambo-toc-target]"),
    );
    const firstHeading = headings[0];
    if (!firstHeading || links.length === 0) return;

    let animationFrame = 0;
    const update = () => {
      animationFrame = 0;
      if (navigation.offsetParent === null) return;

      const threshold = Number.parseFloat(getComputedStyle(firstHeading).scrollMarginTop) || 0;
      const pageHeight = Math.max(
        document.body.scrollHeight,
        document.documentElement.scrollHeight,
      );
      const atBottom = Math.ceil(window.scrollY + window.innerHeight)
        >= pageHeight - 1;
      const index = activeHeadingIndex(
        headings.map((heading) => heading.getBoundingClientRect().top),
        threshold,
        atBottom,
      );
      const activeId = headings[index]?.id;
      let activeLink: HTMLAnchorElement | undefined;

      for (const link of links) {
        const active = link.dataset.mamboTocTarget === activeId;
        if (active) {
          link.setAttribute("aria-current", "location");
          if (link.offsetParent !== null) activeLink ??= link;
        } else {
          link.removeAttribute("aria-current");
        }
      }

      if (!activeLink || navigation.scrollHeight <= navigation.clientHeight) return;
      const navigationRect = navigation.getBoundingClientRect();
      const linkRect = activeLink.getBoundingClientRect();
      if (linkRect.top < navigationRect.top) {
        navigation.scrollTop -= navigationRect.top - linkRect.top;
      } else if (linkRect.bottom > navigationRect.bottom) {
        navigation.scrollTop += linkRect.bottom - navigationRect.bottom;
      }
    };
    const schedule = () => {
      if (animationFrame === 0) animationFrame = window.requestAnimationFrame(update);
    };

    schedule();
    window.addEventListener("scroll", schedule, { passive: true });
    window.addEventListener("resize", schedule);
    window.addEventListener("hashchange", schedule);
    navigation.addEventListener("toggle", schedule, true);
    return () => {
      if (animationFrame !== 0) window.cancelAnimationFrame(animationFrame);
      window.removeEventListener("scroll", schedule);
      window.removeEventListener("resize", schedule);
      window.removeEventListener("hashchange", schedule);
      navigation.removeEventListener("toggle", schedule, true);
    };
  }, [headingIds]);

  return <span ref={marker} hidden />;
}

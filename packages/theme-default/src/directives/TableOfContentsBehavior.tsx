"use client";

import { useEffect, useRef } from "react";

export function activeHeadingIndex(
  headingTops: readonly number[],
  threshold: number,
) {
  if (headingTops.length === 0) return -1;

  let active = -1;
  for (let index = 0; index < headingTops.length; index += 1) {
    if ((headingTops[index] ?? Number.POSITIVE_INFINITY) > threshold) break;
    active = index;
  }
  return active;
}

export function headingIndexForScrollIntent(
  geometricIndex: number,
  activeIndex: number,
  direction: -1 | 1,
  headingCount: number,
) {
  if (direction < 0) return geometricIndex;
  if (activeIndex < geometricIndex) return geometricIndex;
  return Math.min(activeIndex + 1, headingCount - 1);
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
    let activeIndex = -1;
    let bottomIndex: number | undefined;
    let intentDistance = 0;
    let previousScrollY = window.scrollY;
    let previousTouchY: number | undefined;
    const atPageBottom = () => {
      const pageHeight = Math.max(
        document.body.scrollHeight,
        document.documentElement.scrollHeight,
      );
      return Math.ceil(window.scrollY + window.innerHeight) >= pageHeight - 1;
    };
    const geometricIndex = () => activeHeadingIndex(
      headings.map((heading) => heading.getBoundingClientRect().top),
      Number.parseFloat(getComputedStyle(firstHeading).scrollMarginTop) || 0,
    );
    const show = (index: number) => {
      activeIndex = index;
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
    const update = () => {
      animationFrame = 0;
      if (navigation.offsetParent === null) return;

      const currentScrollY = window.scrollY;
      if (currentScrollY < previousScrollY || !atPageBottom()) bottomIndex = undefined;
      previousScrollY = currentScrollY;
      show(bottomIndex ?? geometricIndex());
    };
    const schedule = () => {
      if (animationFrame === 0) animationFrame = window.requestAnimationFrame(update);
    };
    const handleIntent = (direction: -1 | 1, distance: number) => {
      if (direction < 0) {
        intentDistance = 0;
        bottomIndex = undefined;
        if (navigation.offsetParent !== null) {
          show(headingIndexForScrollIntent(geometricIndex(), activeIndex, direction, headings.length));
        }
        schedule();
        return;
      }
      if (navigation.offsetParent === null || !atPageBottom()) return;

      intentDistance += distance;
      if (intentDistance < 40) return;
      intentDistance = 0;
      bottomIndex = headingIndexForScrollIntent(
        geometricIndex(),
        bottomIndex ?? activeIndex,
        direction,
        headings.length,
      );
      show(bottomIndex);
    };
    const handleWheel = (event: WheelEvent) => {
      if (event.target instanceof Node && navigation.contains(event.target)) return;
      if (event.deltaY === 0) return;
      const distance = Math.abs(event.deltaY)
        * (event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? window.innerHeight : 1);
      handleIntent(event.deltaY < 0 ? -1 : 1, distance);
    };
    const handleTouchStart = (event: TouchEvent) => {
      if (event.target instanceof Node && navigation.contains(event.target)) {
        previousTouchY = undefined;
        return;
      }
      previousTouchY = event.touches[0]?.clientY;
    };
    const handleTouchMove = (event: TouchEvent) => {
      if (event.target instanceof Node && navigation.contains(event.target)) return;
      const currentTouchY = event.touches[0]?.clientY;
      if (previousTouchY === undefined || currentTouchY === undefined) return;
      const delta = previousTouchY - currentTouchY;
      previousTouchY = currentTouchY;
      if (delta !== 0) handleIntent(delta < 0 ? -1 : 1, Math.abs(delta));
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.altKey || event.ctrlKey || event.metaKey) return;
      if (event.target instanceof Element
        && event.target.closest("a, button, input, select, summary, textarea, [contenteditable]")) return;
      const direction = event.key === "ArrowUp" || event.key === "PageUp" || event.key === "Home"
        || (event.key === " " && event.shiftKey)
        ? -1
        : event.key === "ArrowDown" || event.key === "PageDown" || event.key === "End"
          || event.key === " "
          ? 1
          : 0;
      if (direction !== 0) handleIntent(direction, 40);
    };
    const reset = () => {
      bottomIndex = undefined;
      intentDistance = 0;
      schedule();
    };

    schedule();
    window.addEventListener("scroll", schedule, { passive: true });
    window.addEventListener("wheel", handleWheel, { passive: true });
    window.addEventListener("touchstart", handleTouchStart, { passive: true });
    window.addEventListener("touchmove", handleTouchMove, { passive: true });
    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("resize", reset);
    window.addEventListener("hashchange", reset);
    navigation.addEventListener("toggle", schedule, true);
    return () => {
      if (animationFrame !== 0) window.cancelAnimationFrame(animationFrame);
      window.removeEventListener("scroll", schedule);
      window.removeEventListener("wheel", handleWheel);
      window.removeEventListener("touchstart", handleTouchStart);
      window.removeEventListener("touchmove", handleTouchMove);
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("resize", reset);
      window.removeEventListener("hashchange", reset);
      navigation.removeEventListener("toggle", schedule, true);
    };
  }, [headingIds]);

  return <span ref={marker} hidden />;
}

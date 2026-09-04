"use client";

import { useEffect } from "react";

export function PageBackBehavior({ targetId }: { readonly targetId: string }) {
  useEffect(() => {
    const target = document.getElementById(targetId);
    if (!target) return;

    const goBack = (event: MouseEvent) => {
      if (
        event.defaultPrevented ||
        event.button !== 0 ||
        event.metaKey ||
        event.ctrlKey ||
        event.shiftKey ||
        event.altKey ||
        window.history.length <= 1
      ) return;
      const clicked = event.target;
      if (!(clicked instanceof Element) || !clicked.closest("[data-mambo-back] a")) return;
      event.preventDefault();
      window.history.back();
    };

    target.addEventListener("click", goBack);
    return () => target.removeEventListener("click", goBack);
  }, [targetId]);

  return null;
}

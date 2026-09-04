"use client";

import { useEffect } from "react";

export function PageBackBehavior({ targetId }: { readonly targetId: string }) {
  useEffect(() => {
    const target = document.getElementById(targetId);
    if (!target) return;

    const handleNavigation = (event: MouseEvent) => {
      if (
        event.defaultPrevented ||
        event.button !== 0 ||
        event.metaKey ||
        event.ctrlKey ||
        event.shiftKey ||
        event.altKey
      ) return;
      const clicked = event.target;
      if (!(clicked instanceof Element)) return;
      const fragment = clicked.closest('a[href^="#"]') as HTMLAnchorElement | null;
      if (fragment?.hash) {
        event.preventDefault();
        window.location.replace(fragment.href);
        return;
      }
      if (!clicked.closest("[data-mambo-back] a") || window.history.length <= 1) return;
      event.preventDefault();
      window.history.back();
    };

    target.addEventListener("click", handleNavigation);
    return () => target.removeEventListener("click", handleNavigation);
  }, [targetId]);

  return null;
}

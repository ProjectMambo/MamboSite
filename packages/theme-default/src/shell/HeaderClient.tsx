"use client";

import { useEffect, useState } from "react";

export interface HeaderBehaviorProps {
  readonly targetId: string;
}

/** Adds scroll behaviour to the server-rendered header without owning its links. */
export function HeaderBehavior({ targetId }: HeaderBehaviorProps) {
  useEffect(() => {
    const header = document.getElementById(targetId);
    if (!header) return;
    let previous = window.scrollY;
    const updateVisibility = () => {
      const styles = window.getComputedStyle(document.documentElement);
      const enabled = styles.getPropertyValue("--mambo-header-hide-on-scroll").trim() !== "0";
      const threshold = Number.parseFloat(
        styles.getPropertyValue("--mambo-header-hide-after").trim(),
      ) || 80;
      const current = window.scrollY;
      header.classList.toggle(
        "mambo-site-header--hidden",
        enabled && current > previous && current > threshold,
      );
      previous = current;
    };
    window.addEventListener("scroll", updateVisibility, { passive: true });
    return () => window.removeEventListener("scroll", updateVisibility);
  }, [targetId]);

  return null;
}

export interface ThemeButtonProps {
  readonly defaultScheme: string;
  readonly schemes: readonly string[];
}

export function ThemeButton({ defaultScheme, schemes }: ThemeButtonProps) {
  function toggleTheme() {
    const available = schemes.length > 0 ? schemes : [defaultScheme];
    const current = document.documentElement.dataset.theme ?? defaultScheme;
    const currentIndex = available.indexOf(current);
    const next = available[(currentIndex + 1 + available.length) % available.length] ?? defaultScheme;
    document.documentElement.dataset.theme = next;
    window.localStorage.setItem("mambo-theme", next);
  }

  return (
    <button
      aria-label="Change colour theme"
      className="mambo-theme-toggle"
      onClick={toggleTheme}
      title="Change colour theme"
      type="button"
    >
      <span aria-hidden="true">◐</span>
    </button>
  );
}

export function SiteClock({ locale }: { readonly locale: string }) {
  const [clock, setClock] = useState("");

  useEffect(() => {
    const updateClock = () => {
      setClock(new Intl.DateTimeFormat(locale, {
        day: "2-digit",
        month: "short",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
        hour12: false,
      }).format(new Date()));
    };
    updateClock();
    const timer = window.setInterval(updateClock, 1000);
    return () => window.clearInterval(timer);
  }, [locale]);

  return <time className="mambo-site-clock" suppressHydrationWarning>{clock}</time>;
}

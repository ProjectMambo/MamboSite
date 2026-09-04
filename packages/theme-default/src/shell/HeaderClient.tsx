"use client";

import {
  Children,
  cloneElement,
  useEffect,
  useId,
  useRef,
  useState,
  type KeyboardEvent,
  type ReactElement,
  type ReactNode,
} from "react";

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

export function NavigationMenu({ children }: { readonly children: ReactNode }) {
  const [open, setOpen] = useState(false);
  const navigationId = useId();
  const toggleRef = useRef<HTMLButtonElement>(null);

  function handleKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    if (event.key !== "Escape") return;
    setOpen(false);
    toggleRef.current?.focus();
  }

  if (Children.count(children) === 0) {
    return <div className="mambo-site-navigation-menu" />;
  }

  return (
    <div className="mambo-site-navigation-menu" onKeyDown={handleKeyDown}>
      <button
        aria-controls={navigationId}
        aria-expanded={open}
        aria-label={`${open ? "Close" : "Open"} primary navigation`}
        className="mambo-navigation-toggle"
        onClick={() => setOpen((current) => !current)}
        ref={toggleRef}
        type="button"
      >
        <span aria-hidden="true">{open ? "×" : "☰"}</span>
      </button>
      <nav
        aria-label="Primary navigation"
        className="mambo-site-navigation"
        data-open={open}
        id={navigationId}
        onClick={() => setOpen(false)}
      >
        {children}
      </nav>
    </div>
  );
}

export interface TooltipProps {
  readonly children: ReactElement<{ "aria-describedby"?: string }>;
  readonly label: string;
}

export function Tooltip({ children, label }: TooltipProps) {
  const id = useId();
  const existingDescription = children.props["aria-describedby"];
  const describedBy = existingDescription ? `${existingDescription} ${id}` : id;
  return (
    <span className="mambo-tooltip">
      {cloneElement(children, { "aria-describedby": describedBy })}
      <span className="mambo-tooltip__content" id={id} role="tooltip">{label}</span>
    </span>
  );
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
    <Tooltip label="Change colour theme">
      <button
        aria-label="Change colour theme"
        className="mambo-theme-toggle"
        onClick={toggleTheme}
        type="button"
      >
        <span aria-hidden="true">◐</span>
      </button>
    </Tooltip>
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

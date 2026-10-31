import { useEffect, useRef } from "react";
import { componentCount } from "./data";

/** Search: Slash focuses it, Enter opens the first match. */
export function Find({ query, onQuery, onEnter, className }: { query: string; onQuery: (q: string) => void; onEnter: () => void; className: string }) {
  const field = useRef<HTMLInputElement>(null);
  useEffect(() => {
    const slash = (e: KeyboardEvent) => {
      if (e.key !== "/" || e.metaKey || e.ctrlKey || typing(e) || !field.current?.checkVisibility()) return;
      e.preventDefault();
      field.current?.focus();
    };
    addEventListener("keydown", slash);
    return () => removeEventListener("keydown", slash);
  }, []);
  return (
    <input
      ref={field}
      className={className}
      type="search"
      placeholder={`Search ${componentCount} components`}
      aria-label="Search components"
      value={query}
      onChange={(e) => onQuery(e.target.value)}
      onKeyDown={(e) => {
        if (e.key === "Enter") onEnter();
        if (e.key === "Escape") onQuery("");
      }}
    />
  );
}

/** Whether a key went to a field, not the page. */
export function typing(e: KeyboardEvent): boolean {
  const t = e.target as HTMLElement;
  return t.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(t.tagName);
}

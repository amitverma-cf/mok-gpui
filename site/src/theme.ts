import { useSyncExternalStore } from "react";

export type Mode = "light" | "dark";

const listeners = new Set<() => void>();

export function mode(): Mode {
  return document.documentElement.classList.contains("dark") ? "dark" : "light";
}

export function setMode(next: Mode): void {
  document.documentElement.classList.toggle("dark", next === "dark");
  try {
    localStorage.setItem("mok-theme", next);
  } catch (error) {
    console.warn("theme: could not keep the mode", error);
  }
  listeners.forEach((listen) => listen());
}

function subscribe(listen: () => void): () => void {
  listeners.add(listen);
  return () => listeners.delete(listen);
}

/** The site's mode, redrawn when it turns. */
export function useMode(): Mode {
  return useSyncExternalStore(subscribe, mode);
}

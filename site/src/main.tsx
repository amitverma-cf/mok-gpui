import "@fontsource-variable/geist";
import "@fontsource-variable/geist-mono";
import { createRootRoute, createRoute, createRouter, redirect, RouterProvider } from "@tanstack/react-router";
import { createRoot } from "react-dom/client";
import { Shell } from "./Shell";
import { chapters } from "./data";
import { Chapters } from "./pages/Chapters";
import { Docs } from "./pages/Docs";
import { Home } from "./pages/Home";
import { NotFound } from "./pages/NotFound";
import { Story } from "./pages/Story";
import "./styles.css";

const root = createRootRoute({ component: Shell, notFoundComponent: () => <NotFound /> });
const home = createRoute({ getParentRoute: () => root, path: "/", component: Home });
const index = createRoute({
  getParentRoute: () => root,
  path: "/components",
  validateSearch: (search: Record<string, unknown>): { chapter?: number } => ({
    chapter: search.chapter ? Number(search.chapter) : undefined,
  }),
  component: Chapters,
});
const chapter = createRoute({
  getParentRoute: () => root,
  path: "/components/$page",
  // A chapter opens its first story.
  beforeLoad: ({ params }) => {
    const found = chapters.find((c) => c.slug === params.page);
    if (found) throw redirect({ to: "/components/$page/$story/", params: { page: found.slug, story: found.stories[0].slug } });
  },
  component: () => <NotFound what="No chapter by that name." />,
});
const story = createRoute({ getParentRoute: () => root, path: "/components/$page/$story", component: Story });
const docs = createRoute({ getParentRoute: () => root, path: "/docs", component: Docs });
const doc = createRoute({ getParentRoute: () => root, path: "/docs/$page", component: Docs });

const basepath = (import.meta.env.BASE_URL || "/").replace(/\/$/, "");

const router = createRouter({
  routeTree: root.addChildren([home, index, chapter, story, docs, doc]),
  basepath,
  trailingSlash: "always",
  scrollRestoration: true,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

// Background prefetch wasm binary into CacheStorage when browser is idle
if (typeof window !== "undefined" && "caches" in window) {
  const prefetchWasm = () => {
    caches.open("mok-gallery-wasm-v1").then(async (cache) => {
      const base = import.meta.env.BASE_URL || "/";
      const prefix = base.endsWith("/") ? base : `${base}/`;
      const wasmUrl = `${prefix}gallery/gallery_bg.wasm`;
      const match = await cache.match(wasmUrl);
      if (!match) {
        fetch(wasmUrl)
          .then((resp) => {
            if (resp.ok) cache.put(wasmUrl, resp);
          })
          .catch(() => {});
      }
    }).catch(() => {});
  };

  if ("requestIdleCallback" in window) {
    (window as any).requestIdleCallback(prefetchWasm, { timeout: 4000 });
  } else {
    setTimeout(prefetchWasm, 2000);
  }
}

createRoot(document.getElementById("root")!).render(<RouterProvider router={router} />);

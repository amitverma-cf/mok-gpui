import manifest from "../../examples/gallery/stories.json";

export interface Story {
  title: string;
  /** The first component alone, as pages show it. */
  name: string;
  slug: string;
  components: string[];
}

export interface Chapter {
  number: number;
  slug: string;
  title: string;
  summary: string;
  stories: Story[];
}

export const chapters: Chapter[] = manifest.pages.map((c) => ({
  ...c,
  stories: c.stories.map((s) => ({ ...s, name: s.title.split(/ \/ | · | → | \(/)[0].trim() })),
}));

/** Components the gallery shows live, one story each. */
export const liveCount = new Set(chapters.flatMap((c) => c.stories.flatMap((s) => s.components))).size;

/** Every story in reading order, for paging and search. */
export const all = chapters.flatMap((chapter) => chapter.stories.map((story) => ({ chapter, story })));

/** Chapters by title, else their stories by title or component. */
export function search(query: string): Chapter[] {
  const q = query.trim().toLowerCase();
  if (!q) return chapters;
  return chapters.flatMap((c) => {
    if (c.title.toLowerCase().includes(q)) return [c];
    const stories = c.stories.filter((s) => s.title.toLowerCase().includes(q) || s.components.some((n) => n.toLowerCase().includes(q)));
    return stories.length ? [{ ...c, stories }] : [];
  });
}

import registryManifest from "../../registry.json";

export interface RegistryComponent {
  name: string;
  display_name: string;
  chapter: number;
  chapter_title: string;
  category: string;
  source_dir: string;
  file: string;
  export_type: string;
  description: string;
  dependencies: string[];
  /** `component` and `feature` copy with `cargo mok add`; the rest say where else they live. */
  status: "component" | "feature" | "core" | "native" | "blocked" | "host" | "unmapped";
  unit?: string;
  import?: string;
  /** Files `cargo mok add` copies for it. */
  files?: number;
}

export interface RegistryChapter {
  number: number;
  name: string;
  title: string;
  summary: string;
  source_dir: string;
  component_count: number;
  components: string[];
}

export const registryComponents = registryManifest.components as RegistryComponent[];
export const registryChapters: RegistryChapter[] = registryManifest.chapters;

/** Every component in the registry, whatever its status. */
export const componentCount = registryComponents.length;

/** The story that draws the component: its own chapter's first, else any chapter's. */
export function getStoryUrl(chapterNumber: number, componentName: string): { page: string; story: string } {
  const plain = (name: string) => name.toLowerCase().replace(/[^a-z0-9]/g, "");
  const draws = (st: { components: string[] }) => st.components.some((n) => plain(n) === plain(componentName));
  const own = chapters.find((c) => c.number === chapterNumber);
  const ownStory = own?.stories.find(draws);
  if (own && ownStory) return { page: own.slug, story: ownStory.slug };
  for (const c of chapters) {
    const found = c.stories.find(draws);
    if (found) return { page: c.slug, story: found.slug };
  }
  throw new Error(`no story draws ${componentName}`);
}

export function getComponentImageSrc(page: string, story: string, mode: string): string {
  let key = `${page}-${story}`;
  if (key.length > 70) {
    key = key.slice(0, 70);
  }
  const base = import.meta.env.BASE_URL || "/";
  const prefix = base.endsWith("/") ? base : `${base}/`;
  return `${prefix}components/${key}-${mode}.webp`;
}

export const REPO = "https://github.com/amitverma-cf/mok-gpui";
export const ORIGINAL_REPO = "https://git.zacharyzhang.com/ZacharyZhang-NY/Ely-GPUI-Components";

export const pad = (n: number) => String(n).padStart(2, "0");

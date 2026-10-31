// Fails unless every registry component resolves to a story whose real capture exists, light and dark.
// Usage: node scripts/captures.mjs   (run from anywhere; reads ../registry.json and the stories)
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const registry = JSON.parse(readFileSync(join(root, "../registry.json"), "utf8"));
const stories = JSON.parse(readFileSync(join(root, "../examples/gallery/stories.json"), "utf8"));
const plain = (name) => name.toLowerCase().replace(/[^a-z0-9]/g, "");
// As the gallery's capture and data.ts name a file: page and story slugs, cut at 70.
const stem = (page, story) => `${page}-${story}`.slice(0, 70);

const problems = [];
for (const c of registry.components) {
  const draws = (story) => story.components.some((n) => plain(n) === plain(c.display_name));
  const own = stories.pages.find((p) => p.number === c.chapter);
  let page = own;
  let story = own.stories.find(draws);
  if (!story) {
    for (const p of stories.pages) {
      story = p.stories.find(draws);
      if (story) {
        page = p;
        break;
      }
    }
  }
  if (!story) {
    problems.push(`${c.name}: no story draws it`);
    continue;
  }
  for (const mode of ["light", "dark"]) {
    const file = `${stem(page.slug, story.slug)}-${mode}.webp`;
    if (!existsSync(join(root, "public/components", file))) problems.push(`${c.name}: ${file} is missing`);
  }
}
if (problems.length) {
  console.error(problems.join("\n"));
  console.error(`${problems.length} problems: add the story, then shoot it with --capture-stories`);
  process.exit(1);
}
console.log(`[ok] all ${registry.components.length} components have a real capture, light and dark`);

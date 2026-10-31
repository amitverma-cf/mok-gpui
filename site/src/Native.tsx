import shots from "./shots.json";
import { useMode } from "./theme";

const captured = new Set<string>(shots.stories);
const windows: Record<string, string[]> = shots.windows;

/** Whether the site shows this story as a native capture. */
export const isCaptured = (page: string, story: string) => captured.has(`${page}/${story}`);

/** A capture at twice its pixels, in the site's mode. */
function Shot({ name, alt }: { name: string; alt: string }) {
  const theme = useMode();
  return <img className="shot" srcSet={`/shots/${name}-${theme}.jpg 2x`} alt={alt} loading="lazy" />;
}

export function Capture({ page, story, title }: { page: string; story: string; title: string }) {
  return (
    <figure className="capture">
      <Shot name={`${page}--${story}`} alt={`${title}, captured from the native gallery`} />
      <figcaption>Native capture, macOS</figcaption>
    </figure>
  );
}

/** The windows a live story opens, captured natively. */
export function Windows({ page, story, title }: { page: string; story: string; title: string }) {
  const names = windows[`${page}/${story}`];
  if (!names) return null;
  return (
    <figure className="capture windows">
      <div className="shots">
        {names.map((name) => (
          <Shot key={name} name={name} alt={`A window ${title} opens, captured natively`} />
        ))}
      </div>
      <figcaption>The windows it opens, natively</figcaption>
    </figure>
  );
}

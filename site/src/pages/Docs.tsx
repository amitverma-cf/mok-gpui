import { Link, useNavigate, useParams } from "@tanstack/react-router";
import { useEffect, type MouseEvent } from "react";
import type { Guide } from "../docs/guides";
import { guides } from "virtual:guides";
import { NotFound } from "./NotFound";

const to = (guide: Guide) => (guide.slug ? `/docs/${guide.slug}/` : "/docs/");

async function copy(button: HTMLButtonElement) {
  const code = button.parentElement?.querySelector("code")?.textContent;
  if (code == null) throw new Error("docs: a copy button without code");
  try {
    await navigator.clipboard.writeText(code);
    button.textContent = "Copied";
  } catch (error) {
    console.error("docs: copy failed", error);
    button.textContent = "Copy failed";
  }
  setTimeout(() => (button.textContent = "Copy"), 1600);
}

export function Docs() {
  const { page } = useParams({ strict: false });
  const navigate = useNavigate();
  const at = guides.findIndex((g) => g.slug === (page ?? ""));
  const guide = guides[at];
  useEffect(() => {
    if (guide) document.title = `${guide.title} · Docs · Mok`;
  }, [guide]);
  if (!guide) return <NotFound what={`No guide at ${page}.`} />;
  const [prev, next] = [guides[at - 1], guides[at + 1]];

  // Links and copy buttons inside the rendered Markdown.
  const press = (e: MouseEvent<HTMLElement>) => {
    const target = e.target as HTMLElement;
    const button = target.closest<HTMLButtonElement>("button.copy");
    if (button) return void copy(button);
    const link = target.closest<HTMLAnchorElement>("a[href^='/']:not([href^='//'])");
    if (!link || e.metaKey || e.ctrlKey || e.shiftKey || e.button !== 0) return;
    e.preventDefault();
    const url = new URL(link.href);
    navigate({ to: url.pathname, hash: url.hash.slice(1) || undefined });
  };

  return (
    <div className="docs">
      <nav className="docs-rail" aria-label="Guides" data-lenis-prevent>
        <ol>
          {guides.map((g) => (
            <li key={g.slug}>
              <Link to={to(g)} activeOptions={{ exact: true }}>
                {g.title}
              </Link>
            </li>
          ))}
        </ol>
      </nav>
      <article className="doc">
        <h1>{guide.title}</h1>
        <div className="prose" onClick={press} dangerouslySetInnerHTML={{ __html: guide.html }} />
        <nav className="pager" aria-label="More guides">
          {prev && <Link to={to(prev)}>← {prev.title}</Link>}
          {next && (
            <Link className="next" to={to(next)}>
              {next.title} →
            </Link>
          )}
        </nav>
      </article>
      {guide.toc.length > 0 && (
        <nav className="doc-toc" aria-label="On this page" data-lenis-prevent>
          <ol>
            {guide.toc.map((h) => (
              <li key={h.id}>
                <a href={`#${h.id}`}>{h.text}</a>
              </li>
            ))}
          </ol>
        </nav>
      )}
    </div>
  );
}

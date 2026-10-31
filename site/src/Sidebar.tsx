import { Link, useNavigate, useParams } from "@tanstack/react-router";
import { useEffect, useMemo, useState } from "react";
import { Find } from "./Find";
import { type Chapter, chapters, pad, registryChapters, registryComponents } from "./data";

export function Sidebar({
  activeChapter,
  onSelectChapter,
  query: externalQuery,
  onQuery: externalOnQuery,
}: {
  activeChapter?: number | null;
  onSelectChapter?: (chapter: number | null) => void;
  query?: string;
  onQuery?: (q: string) => void;
}) {
  const navigate = useNavigate();
  const params = useParams({ strict: false }) as { page?: string; story?: string };
  const [localQuery, setLocalQuery] = useState("");
  const [expanded, setExpanded] = useState<Record<number, boolean>>({});

  const query = externalQuery !== undefined ? externalQuery : localQuery;
  const setQuery = externalOnQuery || setLocalQuery;

  // Resolve which chapter is currently active based on URL page param or activeChapter prop
  const currentChapter = chapters.find((c) => c.slug === params.page);
  const currentChapterNum = currentChapter?.number ?? (activeChapter ? Number(activeChapter) : null);

  const isChapterExpanded = (chNumber: number, chSlug: string) => {
    if (query.trim()) return true;
    if (expanded[chNumber] !== undefined) return expanded[chNumber];
    if (params.page && params.page === chSlug) return true;
    if (currentChapterNum !== null && Number(currentChapterNum) === Number(chNumber)) return true;
    return false;
  };

  const toggleChapter = (chNumber: number, chSlug: string) => {
    setExpanded((prev) => ({
      ...prev,
      [chNumber]: !isChapterExpanded(chNumber, chSlug),
    }));
  };

  const handleOpenChapter = (chNumber: number) => {
    if (onSelectChapter) {
      onSelectChapter(chNumber);
    } else {
      navigate({
        to: "/components/",
        search: { chapter: chNumber } as any,
      });
    }
  };

  const handleAllChaptersClick = () => {
    if (onSelectChapter) {
      onSelectChapter(null);
    } else {
      navigate({ to: "/components/" });
    }
  };

  // Scroll active story into view in sidebar
  useEffect(() => {
    if (params.story) {
      const activeEl = document.querySelector(".sidebar-story-link.active");
      if (activeEl) {
        activeEl.scrollIntoView({ block: "nearest", behavior: "smooth" });
      }
    }
  }, [params.page, params.story]);

  const chapterCounts = useMemo(() => {
    const map = new Map<number, number>();
    for (const r of registryChapters) {
      map.set(r.number, r.component_count);
    }
    return map;
  }, []);

  const filteredChapters = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return chapters;
    return chapters
      .map((ch) => {
        const titleMatch = ch.title.toLowerCase().includes(q) || ch.summary.toLowerCase().includes(q);
        const matchingStories = ch.stories.filter(
          (s) =>
            s.name.toLowerCase().includes(q) ||
            s.title.toLowerCase().includes(q) ||
            s.components.some((c) => c.toLowerCase().includes(q))
        );
        if (titleMatch || matchingStories.length > 0) {
          return { ...ch, stories: titleMatch ? ch.stories : matchingStories };
        }
        return null;
      })
      .filter((ch): ch is Chapter => ch !== null);
  }, [query]);

  const isAllActive = !params.page && (activeChapter === null || activeChapter === undefined);

  return (
    <aside className="components-sidebar" data-lenis-prevent>
      <div className="sidebar-search-wrap">
        <Find
          className="find small"
          query={query}
          onQuery={setQuery}
          onEnter={() => {}}
        />
      </div>

      <div className="sidebar-chapter-list">
        <button
          type="button"
          className={`chapter-nav-item all-chapters ${isAllActive ? "active" : ""}`}
          onClick={handleAllChaptersClick}
        >
          <span className="chapter-chevron-spacer" />
          <span className="chapter-nav-title">All Chapters</span>
          <span className="chapter-nav-badge">{registryComponents.length}</span>
        </button>

        <div className="sidebar-divider" />

        {filteredChapters.map((ch) => {
          const isExpanded = isChapterExpanded(ch.number, ch.slug);
          const isChapterActive =
            (params.page && params.page === ch.slug) ||
            (!params.page && activeChapter !== null && Number(activeChapter) === Number(ch.number));

          return (
            <div key={ch.number} className="sidebar-chapter-group">
              <div className={`chapter-nav-item ${isChapterActive ? "active" : ""}`}>
                {/* Chevron icon button ONLY opens/closes accordion */}
                <button
                  type="button"
                  className={`chapter-chevron-btn ${isExpanded ? "open" : ""}`}
                  onClick={(e) => {
                    e.stopPropagation();
                    toggleChapter(ch.number, ch.slug);
                  }}
                  aria-label={isExpanded ? `Collapse ${ch.title}` : `Expand ${ch.title}`}
                  title={isExpanded ? "Collapse" : "Expand"}
                >
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                    <polyline points="9 18 15 12 9 6" />
                  </svg>
                </button>

                {/* Text click opens picture/list view for this chapter */}
                <Link
                  to="/components/"
                  search={{ chapter: ch.number } as any}
                  className="chapter-nav-content"
                  onClick={(e) => {
                    if (onSelectChapter) {
                      e.preventDefault();
                      handleOpenChapter(ch.number);
                    }
                  }}
                  title={`Open ${ch.title} components view`}
                >
                  <span className="chapter-nav-num">{pad(ch.number)}</span>
                  <span className="chapter-nav-title">{ch.title}</span>
                  <span className="chapter-nav-badge">
                    {chapterCounts.get(ch.number) ?? ch.stories.length}
                  </span>
                </Link>
              </div>

              {isExpanded && (
                <ul className="sidebar-story-list">
                  {ch.stories.map((s) => {
                    const isStoryActive = params.page === ch.slug && params.story === s.slug;
                    return (
                      <li key={s.slug} className="sidebar-story-item">
                        <Link
                          to="/components/$page/$story/"
                          params={{ page: ch.slug, story: s.slug }}
                          className={`sidebar-story-link ${isStoryActive ? "active" : ""}`}
                          title={s.title}
                        >
                          <span className="sidebar-story-bullet" />
                          <span className="sidebar-story-name">{s.title}</span>
                        </Link>
                      </li>
                    );
                  })}
                </ul>
              )}
            </div>
          );
        })}
      </div>
    </aside>
  );
}

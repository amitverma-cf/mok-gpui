import { useNavigate, useSearch } from "@tanstack/react-router";
import { useEffect, useMemo, useState } from "react";
import { ComponentCard } from "../ComponentCard";
import { Sidebar } from "../Sidebar";
import {
  type RegistryChapter,
  type RegistryComponent,
  pad,
  registryChapters,
  registryComponents,
} from "../data";

export function Chapters() {
  const searchParams = useSearch({ strict: false }) as { chapter?: string | number };
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const [viewMode, setViewMode] = useState<"grid" | "list">("grid");

  const activeChapter = searchParams?.chapter ? Number(searchParams.chapter) : null;

  const handleSelectChapter = (chNum: number | null) => {
    navigate({
      to: "/components/",
      search: (chNum !== null ? { chapter: chNum } : {}) as any,
    });
  };

  useEffect(() => {
    document.title = activeChapter !== null
      ? `${registryChapters.find((c) => Number(c.number) === Number(activeChapter))?.title} · Components · Mok`
      : `Components (${registryComponents.length.toLocaleString("en-US")}) · Mok`;
  }, [activeChapter]);

  // Filter components by search query or chapter
  const filteredComponents = useMemo(() => {
    let list = registryComponents;
    if (activeChapter !== null) {
      list = list.filter((c) => Number(c.chapter) === Number(activeChapter));
    }
    if (query.trim()) {
      const q = query.trim().toLowerCase().replace("-", "_");
      list = list.filter(
        (c) =>
          c.name.toLowerCase().includes(q) ||
          c.display_name.toLowerCase().includes(q) ||
          c.description.toLowerCase().includes(q) ||
          c.category.toLowerCase().includes(q)
      );
    }
    return list;
  }, [activeChapter, query]);

  // Group by chapter
  const displayedChapters = useMemo(() => {
    if (activeChapter !== null) {
      const ch = registryChapters.find((c) => Number(c.number) === Number(activeChapter));
      return ch ? [ch] : [];
    }
    if (query.trim()) {
      // Only include chapters that have matching components
      const activeNums = new Set(filteredComponents.map((c) => Number(c.chapter)));
      return registryChapters.filter((c) => activeNums.has(Number(c.number)));
    }
    return registryChapters;
  }, [activeChapter, query, filteredComponents]);

  const activeChapterData = activeChapter !== null
    ? registryChapters.find((c) => Number(c.number) === Number(activeChapter))
    : null;

  return (
    <div className="components-layout">
      <Sidebar
        activeChapter={activeChapter}
        onSelectChapter={handleSelectChapter}
        query={query}
        onQuery={setQuery}
      />

      {/* Main Content Area */}
      <main className="components-content">
        <header className="components-header">
          <div className="components-header-info">
            <span className="components-header-overline">
              {activeChapterData ? "Chapter Components" : "All Chapters"}
            </span>
            <h1 className="components-header-title">
              {activeChapterData ? activeChapterData.title : "Components"}
            </h1>
            <p className="components-header-desc">
              {activeChapterData
                ? activeChapterData.summary
                : `Interactive picture view of all ${registryComponents.length} components across ${registryChapters.length} chapters.`}
            </p>
          </div>

          <div className="components-header-actions">
            <div className="view-toggle">
              <button
                type="button"
                className={`view-btn ${viewMode === "grid" ? "active" : ""}`}
                onClick={() => setViewMode("grid")}
                title="Picture View (Grid)"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <rect x="3" y="3" width="7" height="7" rx="1" />
                  <rect x="14" y="3" width="7" height="7" rx="1" />
                  <rect x="3" y="14" width="7" height="7" rx="1" />
                  <rect x="14" y="14" width="7" height="7" rx="1" />
                </svg>
                Picture View
              </button>
              <button
                type="button"
                className={`view-btn ${viewMode === "list" ? "active" : ""}`}
                onClick={() => setViewMode("list")}
                title="Dense List View"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <line x1="8" y1="6" x2="21" y2="6" />
                  <line x1="8" y1="12" x2="21" y2="12" />
                  <line x1="8" y1="18" x2="21" y2="18" />
                  <line x1="3" y1="6" x2="3.01" y2="6" />
                  <line x1="3" y1="12" x2="3.01" y2="12" />
                  <line x1="3" y1="18" x2="3.01" y2="18" />
                </svg>
                List
              </button>
            </div>
          </div>
        </header>

        {query && (
          <div className="search-stats-banner">
            Found <strong>{filteredComponents.length}</strong> components matching &ldquo;{query}&rdquo;
          </div>
        )}

        {filteredComponents.length === 0 ? (
          <div className="empty-results-box">
            <p>No components match &ldquo;{query}&rdquo;.</p>
            <button
              type="button"
              className="text"
              onClick={() => {
                setQuery("");
                handleSelectChapter(null);
              }}
            >
              Reset filters
            </button>
          </div>
        ) : (
          <div className="chapters-sections-wrap">
            {displayedChapters.map((ch) => {
              const chComps = filteredComponents.filter(
                (c) => Number(c.chapter) === Number(ch.number)
              );
              if (chComps.length === 0) return null;

              return (
                <ChapterSection
                  key={ch.number}
                  chapter={ch}
                  components={chComps}
                  viewMode={viewMode}
                  standalone={activeChapter !== null}
                />
              );
            })}
          </div>
        )}
      </main>
    </div>
  );
}

/** Chapter section that renders its components directly */
function ChapterSection({
  chapter,
  components,
  viewMode,
  standalone,
}: {
  chapter: RegistryChapter;
  components: RegistryComponent[];
  viewMode: "grid" | "list";
  standalone: boolean;
}) {
  return (
    <section className="chapter-display-section" id={`chapter-${chapter.number}`}>
      {!standalone && (
        <div className="chapter-section-header">
          <div className="chapter-section-badge">{pad(chapter.number)}</div>
          <div>
            <h2 className="chapter-section-title">{chapter.title}</h2>
            <p className="chapter-section-summary">{chapter.summary}</p>
          </div>
          <span className="chapter-section-count">
            {components.length} components
          </span>
        </div>
      )}

      {viewMode === "grid" ? (
        <div className="components-card-grid">
          {components.map((item) => (
            <ComponentCard key={item.name} item={item} />
          ))}
        </div>
      ) : (
        <div className="components-list-table">
          {components.map((item) => (
            <div key={item.name} className="list-row-item">
              <span className="list-col-name">{item.display_name}</span>
              <span className="list-col-desc">{item.description}</span>
              <code className="list-col-type">{item.export_type}</code>
              <code className="list-col-cli">cargo mok add {item.name}</code>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

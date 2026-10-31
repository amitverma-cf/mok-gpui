import { Link } from "@tanstack/react-router";
import { useState } from "react";
import { type RegistryComponent, getComponentImageSrc, getStoryUrl } from "./data";
import { useMode } from "./theme";

const STATUS: Record<string, string> = {
  core: "in the mok-gpui crate",
  native: "native to gpui",
  blocked: "blocked in gpui",
  host: "your app provides it",
  unmapped: "part of the gallery",
};

export function ComponentCard({ item }: { item: RegistryComponent }) {
  const [copied, setCopied] = useState(false);
  const [imgError, setImgError] = useState(false);
  const mode = useMode();
  const target = getStoryUrl(item.chapter, item.display_name);
  const imageSrc = getComponentImageSrc(target.page, target.story, mode);
  const copyable = item.status === "component" || item.status === "feature";

  const copyCommand = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    navigator.clipboard.writeText(`cargo mok add ${item.name.toLowerCase()}`);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="comp-card">
      <Link to="/components/$page/$story/" params={target} className="comp-card-preview" title={`Open ${item.display_name} demo`}>
        {!imgError && (
          <img
            src={imageSrc}
            alt={item.display_name}
            className="comp-card-real-img"
            onError={() => setImgError(true)}
            loading="lazy"
          />
        )}
        <div className="comp-card-hover-overlay">
          <span className="comp-hover-badge">Open Interactive Demo →</span>
        </div>
      </Link>

      <div className="comp-card-body">
        <div className="comp-card-header">
          <Link to="/components/$page/$story/" params={target} className="comp-card-title">
            {item.display_name}
          </Link>
          <span className="comp-card-chap" title={item.chapter_title}>{item.chapter_title}</span>
        </div>

        <p className="comp-card-desc">{item.description}</p>

        <div className="comp-card-footer">
          {copyable ? (
            <button type="button" className={`comp-copy-btn ${copied ? "copied" : ""}`} onClick={copyCommand} title={`Copy 'cargo mok add ${item.name}'`}>
              {copied ? "✓ Copied command" : `cargo mok add ${item.name.toLowerCase()}`}
            </button>
          ) : (
            <span className="comp-copy-btn note" title={item.description}>
              {STATUS[item.status] ?? item.status}
              {item.import ? ` · ${item.import}` : ""}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

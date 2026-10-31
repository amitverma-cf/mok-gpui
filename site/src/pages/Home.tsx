import { Link } from "@tanstack/react-router";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { all, chapters, componentCount, liveCount, REPO } from "../data";
import { Iso, useStill } from "../iso/Iso";
import { BEAT, strike } from "../iso/marimba";
import { app, bars, calendar, candles, chat, editor } from "../iso/models";
import { Live } from "../Live";
import { City } from "./City";

const APP = app();

const FEATURES = [
  { title: "Markets.", line: "Candles, books and tapes, redrawn every frame.", model: candles(), page: "finance", story: "candlestickchart-ohlcchart-heikinashichart-linequotechart-volumechart-charttypeswitcher-timerangeselector-intervalselector" },
  { title: "Charts.", line: "Areas, bars and stacks that answer the pointer.", model: bars(), page: "charts", story: "areachart-stackedarea-barchart" },
  { title: "Conversations.", line: "Bubbles, avatars and the day between them.", model: chat(), page: "chat", story: "chatcontainer-messagelist-messagebubble-messageavatar-messageheader-messagefooter-dateseparator-scrolltobottombutton" },
  { title: "Code.", line: "Folds, minimap, brackets and many cursors.", model: editor(), page: "editor", story: "codeeditor-editorgutter-linenumbers-foldingcontrol-minimap-ruler-indentguide-bracketmatcher-rainbowbrackets-multicursor-selectionhighlight-stickyscroll-breadcrumb-inlayhints-codelens-inlinedecoration" },
  { title: "Time.", line: "A month of days, its events in place.", model: calendar(), page: "calendar", story: "calendarmonthview" },
].map((f) => {
  const story = chapters.find((c) => c.slug === f.page)?.stories.find((s) => s.slug === f.story);
  if (!story) throw new Error(`home: no story ${f.page}/${f.story}`);
  return { ...f, name: story.name };
});

/** Mounts its children while near the view or holding focus. */
function Near({ className, children }: { className: string; children: ReactNode }) {
  const box = useRef<HTMLDivElement>(null);
  const [near, setNear] = useState(false);
  useEffect(() => {
    const el = box.current!;
    let seen = false;
    const view = new IntersectionObserver(
      ([e]) => {
        seen = e.isIntersecting;
        setNear(seen || el.contains(document.activeElement));
      },
      { rootMargin: "100% 0px" },
    );
    const left = (e: FocusEvent) => setNear(seen || el.contains(e.relatedTarget as Node | null));
    view.observe(el);
    el.addEventListener("focusout", left);
    return () => {
      view.disconnect();
      el.removeEventListener("focusout", left);
    };
  }, []);
  return (
    <div ref={box} className={className}>
      {near && children}
    </div>
  );
}

/** Counts up in sixteen beats once it shows. */
function Count({ to }: { to: number }) {
  const box = useRef<HTMLSpanElement>(null);
  const [n, setN] = useState(to);
  const still = useStill();
  useEffect(() => {
    if (still) return setN(to);
    let frame = 0;
    const seen = new IntersectionObserver(
      ([e]) => {
        if (!e.isIntersecting) return;
        seen.disconnect();
        const start = performance.now();
        let step = -1;
        const tick = (now: number) => {
          const next = Math.min(16, Math.floor((now - start) / BEAT));
          if (next !== step) {
            step = next;
            setN(Math.round(to * (1 - (1 - step / 16) ** 3)));
            strike(step, 0.4);
          }
          if (step < 16) frame = requestAnimationFrame(tick);
        };
        frame = requestAnimationFrame(tick);
      },
      { threshold: 0.6 },
    );
    seen.observe(box.current!);
    return () => {
      seen.disconnect();
      cancelAnimationFrame(frame);
    };
  }, [to, still]);
  return <span ref={box}>{n}</span>;
}

export function Home() {
  useEffect(() => {
    document.title = "Mok · GPUI components";
  }, []);
  return (
    <>
      <section className="hero">
        <Iso model={APP} label="A miniature app growing cell by cell" className="hero-iso" />
        <h1>Components for GPUI.</h1>
        <p>
          {componentCount} components in {chapters.length} chapters, {liveCount} of them live in the browser.
        </p>
        <Link to="/components/" className="button">
          Browse
        </Link>
      </section>
      {FEATURES.map((f) => (
        <section className="feature" key={f.page}>
          <div className="feature-words">
            <h2 className="rise">{f.title}</h2>
            <p className="rise">{f.line}</p>
          </div>
          <Iso model={f.model} label={`A miniature of ${f.title.slice(0, -1).toLowerCase()}`} className="feature-iso" />
          <Near className="feature-live">
            <Live page={f.page} story={f.story} title={f.name} />
          </Near>
        </section>
      ))}
      <section className="tally">
        {[
          [componentCount, "components"],
          [all.length, "stories"],
          [chapters.length, "chapters"],
        ].map(([n, label]) => (
          <p className="rise" key={label}>
            <Count to={n as number} />
            <span>{label}</span>
          </p>
        ))}
      </section>
      <City />
      <section className="close">
        <h2 className="rise">Build with Mok.</h2>
        <code className="rise">mok-gpui = {`{ git = "${REPO}" }`}</code>
        <Link to="/components/" className="button rise">
          Browse
        </Link>
      </section>
    </>
  );
}

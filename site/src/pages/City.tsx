import { Link, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { chapters } from "../data";
import { Iso } from "../iso/Iso";
import { city } from "../iso/models";

const CITY = city(chapters.map((c) => c.stories.length));

/** Chapters as plinths; a plinth and its name light together. */
export function City() {
  const navigate = useNavigate();
  // Plinth i + 1 is chapter i; 0 is the table.
  const [at, setAt] = useState<number | null>(null);
  const open = (i: number) => navigate({ to: "/components/$page/", params: { page: chapters[i].slug } });
  return (
    <section className="city">
      <h2 className="rise">{at === null ? `${chapters.length} chapters.` : `${chapters[at].title}.`}</h2>
      <Iso
        model={CITY}
        label="Every chapter a plinth, as tall as its stories are many"
        className="city-iso"
        onHover={(i) => setAt(i === null ? null : i - 1)}
        onPick={(i) => open(i - 1)}
        lift={at === null ? null : at + 1}
      />
      <ol className="city-names" onMouseLeave={() => setAt(null)}>
        {chapters.map((c, i) => (
          <li key={c.slug} className={at === i ? "on" : undefined}>
            <Link to="/components/$page/" params={{ page: c.slug }} onMouseEnter={() => setAt(i)} onFocus={() => setAt(i)} onBlur={() => setAt(null)}>
              {c.title}
            </Link>
          </li>
        ))}
      </ol>
    </section>
  );
}

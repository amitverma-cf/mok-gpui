import { Link } from "@tanstack/react-router";
import { useEffect } from "react";

export function NotFound({ what = "Nothing lives at this address." }: { what?: string }) {
  useEffect(() => {
    console.error(`site: ${what} (${location.pathname})`);
    document.title = "Not found · Mok";
  }, [what]);
  return (
    <section className="missing">
      <h1>Not found</h1>
      <p>{what}</p>
      <Link to="/components/" className="button">
        All components
      </Link>
    </section>
  );
}

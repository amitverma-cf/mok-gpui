import fs from "fs";
import path from "path";

const dist = path.resolve("dist");
const indexHtml = path.join(dist, "index.html");
const notFoundHtml = path.join(dist, "404.html");

if (fs.existsSync(indexHtml)) {
  fs.copyFileSync(indexHtml, notFoundHtml);
  console.log("Copied dist/index.html to dist/404.html for SPA static hosting.");
}

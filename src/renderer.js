"use strict";
const escapeHtml = (value) => value
  .replaceAll("&", "&amp;")
  .replaceAll("<", "&lt;")
  .replaceAll(">", "&gt;")
  .replaceAll('"', "&quot;")
  .replaceAll("'", "&#39;");
const title = escapeHtml(input.title);
const summary = escapeHtml(input.summary);
const status = escapeHtml(input.status_label);
const accent = input.highlighted ? " data-highlighted=\"true\"" : "";
const html = `<article class="service-card"${accent}><h2>${title}</h2><p>${summary}</p><span>${status}</span></article>`;
return JSON.stringify(Object.freeze({
  title: input.title,
  summary: input.summary,
  status_label: input.status_label,
  highlighted: input.highlighted,
  html
}));

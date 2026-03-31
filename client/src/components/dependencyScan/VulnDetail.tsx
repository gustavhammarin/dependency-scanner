import type { OsvVulnerability } from "@/types/dependencyScan";
import { extractFixed, extractRefs, severityClass, severityLabel, truncate } from "./helpers";

export function VulnDetail({ vuln }: { vuln: OsvVulnerability }) {
  const severity = severityLabel(vuln);
  const fixed = extractFixed(vuln);
  const refs = extractRefs(vuln);

  return (
    <div className="rounded-lg border border-border/60 p-3 space-y-1.5">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <p className="font-medium text-sm">{vuln.id}</p>
        {severity && (
          <span className={`rounded border px-2 py-0.5 text-xs font-semibold ${severityClass(severity)}`}>
            {severity}
          </span>
        )}
      </div>

      {vuln.aliases && vuln.aliases.length > 0 && (
        <p className="text-xs text-muted-foreground">
          Aliases: {vuln.aliases.join(", ")}
        </p>
      )}

      {vuln.published && (
        <p className="text-xs text-muted-foreground">
          Published: {vuln.published.slice(0, 10)}
          {vuln.modified && vuln.modified !== vuln.published
            ? ` · Updated: ${vuln.modified.slice(0, 10)}`
            : ""}
        </p>
      )}

      {vuln.database_specific?.cwe_ids && vuln.database_specific.cwe_ids.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {vuln.database_specific.cwe_ids.map((cwe) => (
            <span key={cwe} className="rounded bg-muted px-2 py-0.5 text-xs text-muted-foreground">
              {cwe}
            </span>
          ))}
        </div>
      )}

      {vuln.summary && <p className="text-sm">{vuln.summary}</p>}
      {!vuln.summary && vuln.details && (
        <p className="text-sm text-muted-foreground">{truncate(vuln.details)}</p>
      )}

      <div className="flex flex-wrap gap-2 pt-1">
        {fixed.length > 0
          ? fixed.map((f) => (
              <span key={f} className="rounded bg-green-500/20 px-2 py-0.5 text-xs text-green-300">
                Fix: {f}
              </span>
            ))
          : (
              <span className="rounded bg-amber-500/20 px-2 py-0.5 text-xs text-amber-300">
                No fix listed
              </span>
            )}
      </div>

      {refs.length > 0 && (
        <div className="pt-1 space-y-0.5">
          <p className="text-xs text-muted-foreground">References</p>
          {refs.slice(0, 6).map((r, i) => (
            <a
              key={`${r.url}-${i}`}
              href={r.url}
              target="_blank"
              rel="noopener noreferrer"
              className="flex gap-1.5 text-xs text-blue-400 underline break-all"
            >
              <span className="shrink-0 text-muted-foreground no-underline">[{r.type}]</span>
              {truncate(r.url, 100)}
            </a>
          ))}
        </div>
      )}
    </div>
  );
}

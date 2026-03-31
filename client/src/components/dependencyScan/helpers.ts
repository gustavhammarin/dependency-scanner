import type { OsvVulnerability } from "@/types/dependencyScan";

export function maxSeverity(vulns: OsvVulnerability[]): string | undefined {
  const order = ["CRITICAL", "HIGH", "MODERATE", "MEDIUM", "LOW"];
  for (const level of order) {
    if (vulns.some((v) => severityLabel(v)?.toUpperCase() === level)) return level;
  }
  return undefined;
}
export function severityLabel(vuln: OsvVulnerability): string | undefined {
  return vuln.database_specific?.severity;
}

export function severityClass(label?: string) {
  switch (label?.toUpperCase()) {
    case "CRITICAL": return "bg-red-500/20 text-red-300 border-red-500/30";
    case "HIGH":     return "bg-orange-500/20 text-orange-300 border-orange-500/30";
    case "MODERATE":
    case "MEDIUM":   return "bg-amber-500/20 text-amber-300 border-amber-500/30";
    case "LOW":      return "bg-blue-500/20 text-blue-300 border-blue-500/30";
    default:         return "bg-muted/60 text-muted-foreground border-border";
  }
}

export function extractFixed(vuln: OsvVulnerability): string[] {
  const fixes = vuln.affected?.flatMap((a) =>
    a.ranges?.flatMap((r) =>
      r.events.flatMap((e) =>
        e.fixed ? (a.package?.name ? [`${a.package.name}@${e.fixed}`] : [e.fixed]) : []
      )
    ) ?? []
  ) ?? [];
  return [...new Set(fixes)];
}

export function extractRefs(vuln: OsvVulnerability) {
  const direct = vuln.references?.map((r) => ({ type: r.type ?? "WEB", url: r.url })) ?? [];
  const seen = new Set(direct.map((r) => r.url));
  const extra: { type: string; url: string }[] = [];
  for (const a of vuln.affected ?? []) {
    const src = a.database_specific?.source;
    if (src && !seen.has(src)) { seen.add(src); extra.push({ type: "SOURCE", url: src }); }
  }
  return [...direct, ...extra];
}

export function truncate(s: string, max = 260) {
  return s.length <= max ? s : `${s.slice(0, max)}…`;
}

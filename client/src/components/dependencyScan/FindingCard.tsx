import type { DependencyFinding } from "@/types/dependencyScan";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../ui/card";
import { VulnDetail } from "./VulnDetail";
import { maxSeverity, severityClass } from "./helpers";

export function FindingCard({ finding }: { finding: DependencyFinding }) {
  const top = maxSeverity(finding.vulnerabilities);
  const isDirect = finding.dependency_type === "direct";

  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="space-y-0.5">
            <div className="flex flex-wrap items-center gap-2">
              <CardTitle className="text-base">{finding.name}</CardTitle>
              <span
                className={`rounded border px-2 py-0.5 text-xs font-medium ${
                  isDirect
                    ? "border-violet-500/40 bg-violet-500/15 text-violet-300"
                    : "border-border bg-muted/40 text-muted-foreground"
                }`}
              >
                {isDirect ? "direct" : "transitive"}
              </span>
            </div>
            <CardDescription>
              {finding.ecosystem}
              {finding.version ? ` · ${finding.version}` : ""}
              {" · "}
              <span className="font-mono text-xs">{finding.purl}</span>
            </CardDescription>
          </div>

          {top && (
            <span className={`rounded border px-2 py-1 text-xs font-semibold ${severityClass(top)}`}>
              {top}
            </span>
          )}
        </div>
      </CardHeader>

      <CardContent className="space-y-2">
        {finding.vulnerabilities.map((v) => (
          <VulnDetail key={v.id} vuln={v} />
        ))}
      </CardContent>
    </Card>
  );
}


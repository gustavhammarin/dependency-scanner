import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { useDependencyScan } from "@/hooks/dependencyScan/useDependencyScan";
import { usePackageStore } from "@/hooks/packages/usePackageStore";
import { PackageSelectorCard } from "@/components/PackageSelector";
import { FindingCard } from "@/components/dependencyScan/FindingCard";
import type { PackageSource } from "@/types/source";

function DependencyScanPage() {
  const { loading, scanData, error, fetchDependencyScan } = useDependencyScan();
  const selectedPackage = usePackageStore((s) => s.selectedPackage);

  const handleScan = () => {
    if (!selectedPackage) return;
    fetchDependencyScan({
      package_id: selectedPackage.package_id,
      package_version: selectedPackage.version,
      source: selectedPackage.package_source as PackageSource,
    });
  };

  return (
    <div className="mx-auto w-full max-w-6xl space-y-6">
      <PackageSelectorCard
        header="Dependency Scan"
        description="Scan the dependency graph and correlate vulnerable packages from OSV."
        error={error}
        loading={loading}
        handleClick={handleScan}
      />

      {scanData && (
        <>
          <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
            {[
              { label: "Total scanned", value: scanData.total_scanned },
              { label: "Vulnerable", value: scanData.vulnerable_count },
              { label: "Direct", value: scanData.direct_count },
              { label: "Transitive", value: scanData.transitive_count },
            ].map(({ label, value }) => (
              <Card key={label} className="border-border/60">
                <CardContent className="pt-4 pb-3">
                  <p className="text-2xl font-bold">{value}</p>
                  <p className="text-xs text-muted-foreground mt-0.5">
                    {label}
                  </p>
                </CardContent>
              </Card>
            ))}
          </div>

          {scanData.findings.length === 0 ? (
            <Card>
              <CardHeader>
                <CardTitle>No vulnerable dependencies found</CardTitle>
                <CardDescription>
                  {scanData.total_scanned} packages scanned — all clear.
                </CardDescription>
              </CardHeader>
            </Card>
          ) : (
            <div className="space-y-4">
              {scanData.findings.map((f) => (
                <FindingCard key={f.purl} finding={f} />
              ))}
            </div>
          )}
        </>
      )}
    </div>
  );
}

export default DependencyScanPage;

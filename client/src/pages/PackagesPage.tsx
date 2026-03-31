import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { usePackages, PACKAGES_KEY } from "@/hooks/packages/usePackages";
import { deletePackage, fetchPackage } from "@/api/packages.api";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import type { PackageSource } from "@/types/source";

const SOURCES: PackageSource[] = ["github", "nuget", "npm"];

function PackagesPage() {
  const { data: packages, isLoading, error } = usePackages();
  const queryClient = useQueryClient();

  const [source, setSource] = useState<PackageSource>("nuget");
  const [packageId, setPackageId] = useState("");
  const [version, setVersion] = useState("");
  const [fetching, setFetching] = useState(false);
  const [fetchError, setFetchError] = useState<string | null>(null);
  const [deletingId, setDeletingId] = useState<number | null>(null);

  const handleDelete = async (id: number) => {
    try {
      setDeletingId(id);
      const res = await deletePackage(id);
      if (!res.ok) throw new Error(await res.text() || res.statusText);
      queryClient.invalidateQueries({ queryKey: PACKAGES_KEY });
    } finally {
      setDeletingId(null);
    }
  };

  const handleFetch = async () => {
    if (!packageId || !version) return;
    try {
      setFetching(true);
      setFetchError(null);
      const res = await fetchPackage({ package_id: packageId, version, package_source: source });
      if (!res.ok) {
        const msg = await res.text();
        throw new Error(msg || res.statusText);
      }
      setPackageId("");
      setVersion("");
      queryClient.invalidateQueries({ queryKey: PACKAGES_KEY });
    } catch (e) {
      setFetchError(e instanceof Error ? e.message : "Unknown error");
    } finally {
      setFetching(false);
    }
  };

  return (
    <div className="mx-auto w-full max-w-6xl space-y-6">
      <Card className="border-border/60">
        <CardHeader>
          <CardTitle>Fetch New Package</CardTitle>
          <CardDescription>
            Download a package to make it available for analysis.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex gap-1 rounded-md border border-border p-1 w-fit">
            {SOURCES.map((s) => (
              <button
                key={s}
                type="button"
                onClick={() => setSource(s)}
                className={`rounded px-3 py-1 text-sm transition-colors ${
                  source === s
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:text-foreground"
                }`}
              >
                {s === "github" ? "GitHub" : s === "nuget" ? "NuGet" : "npm"}
              </button>
            ))}
          </div>

          <div className="flex flex-wrap gap-3">
            <Input
              className="flex-1 min-w-48"
              placeholder={source === "github" ? "owner/repository" : "package id"}
              value={packageId}
              onChange={(e) => setPackageId(e.target.value)}
            />
            <Input
              className="flex-1 min-w-32"
              placeholder="version"
              value={version}
              onChange={(e) => setVersion(e.target.value)}
            />
            <Button
              className="w-full sm:w-auto"
              onClick={handleFetch}
              disabled={!packageId || !version || fetching}
            >
              {fetching ? "Fetching…" : "Fetch"}
            </Button>
          </div>

          {fetchError && (
            <p className="text-sm text-red-400">{fetchError}</p>
          )}
        </CardContent>
      </Card>

      <Card className="border-border/60">
        <CardHeader>
          <CardTitle>Downloaded Packages</CardTitle>
          <CardDescription>Packages available for analysis.</CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading && <p className="text-sm text-muted-foreground">Loading…</p>}
          {error && <p className="text-sm text-red-400">{error.message}</p>}
          {packages && packages.length === 0 && (
            <p className="text-sm text-muted-foreground">No packages fetched yet.</p>
          )}
          <div className="space-y-2">
            {packages?.map((p) => (
              <div
                key={p.id}
                className="flex items-center justify-between rounded-md border border-border/40 px-4 py-2 text-sm"
              >
                <span className="font-medium">{p.package_id}</span>
                <div className="flex items-center gap-4 text-muted-foreground">
                  <span>{p.version}</span>
                  <span className="capitalize">{p.package_source}</span>
                  <span>{new Date(p.fetch_date).toLocaleDateString()}</span>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleDelete(Number(p.id))}
                    disabled={deletingId === Number(p.id)}
                    className="text-red-400 hover:text-red-300 hover:bg-red-500/10 h-6 px-2"
                  >
                    {deletingId === Number(p.id) ? "Deleting…" : "Delete"}
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

export default PackagesPage;

import { getDependencyScan } from "@/api/dependencyScan.api";
import type { DependencyScanRequest, DpsScanResult } from "@/types/dependencyScan";
import { useDependencyScanStore } from "./useDependencyScanStore";

export function useDependencyScan() {
  const loading = useDependencyScanStore((state) => state.loading);
  const scanData = useDependencyScanStore((state) => state.scanData);
  const error = useDependencyScanStore((state) => state.error);
  const setLoading = useDependencyScanStore((state) => state.setLoading);
  const setScanData = useDependencyScanStore((state) => state.setScanData);
  const setError = useDependencyScanStore((state) => state.setError);

  const fetchDependencyScan = async (
    payload: DependencyScanRequest,
  ): Promise<DpsScanResult | null> => {
    try {
      setLoading(true);
      setError(null);

      const res = await getDependencyScan(payload);

      if (!res.ok) {
        const errorText = await res.text();
        throw new Error(errorText || res.statusText || "Dependency scan failed");
      }

      const data: DpsScanResult = await res.json();
      setScanData(data);
      return data;
    } catch (error) {
      setScanData(null);
      setError(error instanceof Error ? error.message : "Unknown error");
      return null;
    } finally {
      setLoading(false);
    }
  };

  return { loading, scanData, error, fetchDependencyScan };
}

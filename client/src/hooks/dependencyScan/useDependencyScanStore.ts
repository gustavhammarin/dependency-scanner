import type { DpsScanResult } from "@/types/dependencyScan";
import { create } from "zustand";

type DependencyScanStore = {
  loading: boolean;
  scanData: DpsScanResult | null;
  error: string | null;

  setLoading: (value: boolean) => void;
  setScanData: (value: DpsScanResult | null) => void;
  setError: (value: string | null) => void;
};

export const useDependencyScanStore = create<DependencyScanStore>((set) => ({
  loading: false,
  scanData: null,
  error: null,

  setLoading: (loading) => set({ loading }),
  setScanData: (scanData) => set({ scanData }),
  setError: (error) => set({ error }),
}));

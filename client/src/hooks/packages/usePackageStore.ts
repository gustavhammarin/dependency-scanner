import type { Package } from "@/types/packages"
import { create } from "zustand";

type PackageStore = {
    selectedPackage: Package | null;

    setSelectedPackage: (value: Package | null) => void;
};

export const usePackageStore = create<PackageStore>((set) => ({
    selectedPackage: null,

    setSelectedPackage: (selectedPackage) => set({selectedPackage})
}))

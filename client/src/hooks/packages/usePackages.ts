import { getAllPackages } from "@/api/packages.api"
import type { Package } from "@/types/packages"
import { useQuery } from "@tanstack/react-query"

export const PACKAGES_KEY = ["packages"]

export const usePackages = () => {
    return useQuery<Package[]>({
        queryKey: PACKAGES_KEY,
        queryFn: async () => {
            const res = await getAllPackages();
            if (!res.ok) throw new Error("Failed to fetch all packages");
            return res.json();
        },
    });
};


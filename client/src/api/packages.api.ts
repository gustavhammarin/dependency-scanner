import { apiFetch } from "./client";

export interface FetchPackageRequest {
    package_id: string;
    version: string;
    package_source: string;
}

export const getAllPackages = () => {
    return apiFetch("/packages", {
        method: "GET",
        headers: { "Content-Type": "application/json" },
    });
};

export const fetchPackage = (req: FetchPackageRequest) => {
    return apiFetch("/packages", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(req),
    });
};

export const deletePackage = (id: number) => {
    return apiFetch(`/packages/${id}`, { method: "DELETE" });
};

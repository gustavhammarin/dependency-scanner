import type { DependencyScanRequest } from "@/types/dependencyScan";
import { apiFetch } from "./client";


export function getDependencyScan(payload: DependencyScanRequest) {
  return apiFetch("/dependency-scan", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(payload),
  });
}

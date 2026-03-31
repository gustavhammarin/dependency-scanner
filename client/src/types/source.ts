import type { DependencyScanRequest } from "./dependencyScan";

export const SCAN_SOURCES = ["github", "nuget", "npm"] as const;
export type PackageSource = (typeof SCAN_SOURCES)[number];

export type SourceFieldConfig = {
  key: string;
  placeholder: string;
  required: boolean;
};

export type SourceConfig = {
  label: string;
  fields: SourceFieldConfig[];
  buildRequest: (fields: Record<string, string>) => DependencyScanRequest;
  buildLabel: (fields: Record<string, string>) => string;
};

export const SOURCE_CONFIGS: Record<PackageSource, SourceConfig> = {
  github: {
    label: "GitHub",
    fields: [
      { key: "package_id", placeholder: "owner/repository", required: true},
      { key: "package_version", placeholder: "version", required: true },
    ],
    buildRequest: (f) => ({ source: "github", package_id: f.package_id, package_version: f.package_version }),
    buildLabel: (f) => `Dependency Scan — ${f.owner}/${f.repository}`,
  },
  nuget: {
    label: "NuGet",
    fields: [
      { key: "package_id", placeholder: "package id (e.g. Newtonsoft.Json)", required: true },
      { key: "package_version", placeholder: "version (e.g. 13.0.3)", required: true },
    ],
    buildRequest: (f) => ({ source: "nuget", package_id: f.package_id, package_version: f.package_version }),
    buildLabel: (f) => `Dependency Scan — nuget ${f.package_id} ${f.package_version}`,
  },
  npm: {
    label: "npm",
    fields: [
      { key: "package_id", placeholder: "package name (e.g. lodash)", required: true },
      { key: "package_version", placeholder: "version (e.g. 4.17.21)", required: true },
    ],
    buildRequest: (f) => ({ source: "npm", package_id: f.package_id, package_version: f.package_version }),
    buildLabel: (f) => `Dependency Scan — npm ${f.package_id} ${f.package_version}`,
  },
};

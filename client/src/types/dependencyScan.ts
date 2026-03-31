import type { PackageSource } from "./source";


export interface DependencyScanRequest {
  source: PackageSource;
  package_id: string;
  package_version: string;
}

// ─── Response ────────────────────────────────────────────────────────────────

export interface DpsScanResult {
  total_scanned: number;
  vulnerable_count: number;
  direct_count: number;
  transitive_count: number;
  findings: DependencyFinding[];
}

export type DependencyType = "direct" | "transitive";

export interface DependencyFinding {
  purl: string;
  name: string;
  version: string;
  ecosystem: string;
  dependency_type: DependencyType;
  /** Full OSV vulnerability objects */
  vulnerabilities: OsvVulnerability[];
}

// ─── OSV types (unchanged — backend returns raw OSV API objects) ──────────────

export interface OsvVulnerability {
  id: string;
  aliases?: string[];
  summary?: string;
  details?: string;
  published?: string;
  modified: string;
  severity?: OsvSeverity[];
  database_specific?: OsvVulnerabilityDatabaseSpecific;
  affected?: OsvAffectedPackage[];
  references?: OsvReference[];
}

export interface OsvSeverity {
  type: string;
  score: string;
}

export interface OsvVulnerabilityDatabaseSpecific {
  severity?: string;
  cwe_ids?: string[];
  github_reviewed?: boolean;
}

export interface OsvAffectedPackage {
  package?: { name?: string; ecosystem?: string };
  ranges?: OsvRange[];
  database_specific?: { source?: string };
}

export interface OsvRange {
  type: string;
  events: OsvRangeEvent[];
}

export interface OsvRangeEvent {
  introduced?: string;
  fixed?: string;
  last_affected?: string;
}

export interface OsvReference {
  type: string;
  url: string;
}

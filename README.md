# Dependency Scanner

Scans third-party packages for known vulnerabilities by generating a CycloneDX SBOM and querying the [OSV](https://osv.dev) database. Supports NuGet, npm, and GitHub sources.

## Stack

- **Backend** — Rust (Axum), SQLite
- **Frontend** — React, TypeScript, Vite, Tailwind
- **SBOM** — [cdxgen](https://github.com/CycloneDX/cdxgen)
- **Vulnerabilities** — [OSV API](https://google.github.io/osv.dev/)

## How it works

1. A package is downloaded from NuGet / npm / GitHub
2. A CycloneDX SBOM (`cdx.json`) is generated via `cdxgen`
   - For NuGet: a synthetic `.csproj` is created and `dotnet restore` is run to resolve transitive deps before cdxgen
   - For npm: the tarball is extracted and cdxgen reads `package.json`
   - For GitHub/Python: a venv is created and pip-installed before cdxgen
3. The SBOM is parsed and each component's PURL is sent to the OSV batch API
4. Vulnerable packages are returned with full OSV objects (severity, CVEs, fix versions)
5. Each dependency is classified as **direct** (explicitly pulled in by the scanned package) or **transitive** (pulled in by a direct dep)

Downloaded packages are cached on disk and tracked in SQLite. Cache is capped at 5 GB with FIFO eviction.

## Running locally

### With Docker (recommended)

Copy the example env file and fill in your GitHub token (needed for GitHub source scanning):

```bash
cp example.env .env
```

```bash
docker compose up --build
```

| Service  | URL                    |
|----------|------------------------|
| Frontend | http://localhost:3000  |
| Backend  | http://localhost:8081  |

### Without Docker

**Backend** — requires Rust, dotnet 8 SDK, Node.js, cdxgen, and Python 3:

```bash
cd backend
export DATABASE_URL=sqlite:data.db
cargo run
```

**Frontend:**

```bash
cd client
npm install
npm run dev
```

The frontend dev server proxies to `http://localhost:5000/api` by default.

## API

| Method   | Path                    | Description                        |
|----------|-------------------------|------------------------------------|
| `POST`   | `/api/dependency-scan`  | Run a full scan on a package       |
| `GET`    | `/api/packages`         | List all cached packages           |
| `POST`   | `/api/packages`         | Download and cache a new package   |
| `DELETE` | `/api/packages/{id}`    | Remove a package from cache + disk |

### POST /api/dependency-scan

```json
{
  "package_id": "Newtonsoft.Json",
  "package_version": "13.0.3",
  "source": "nuget"
}
```

`source` is one of `nuget`, `npm`, `github`.

For GitHub, `package_id` is `owner/repository`.

### POST /api/packages

```json
{
  "package_id": "lodash",
  "version": "4.17.21",
  "package_source": "npm"
}
```

## Environment variables

| Variable       | Default            | Description                              |
|----------------|--------------------|------------------------------------------|
| `DATABASE_URL` | `sqlite:data.db`   | SQLite connection string                 |
| `RUST_PORT`    | `5000`             | Backend listen port                      |
| `RUST_LOG`     | `info`             | Log level (`trace`, `debug`, `info`, …)  |
| `GITHUB_API_KEY` | —                | GitHub token — required for GitHub scans |

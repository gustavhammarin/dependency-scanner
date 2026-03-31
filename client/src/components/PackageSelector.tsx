import { usePackages } from "@/hooks/packages/usePackages";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";
import { usePackageStore } from "@/hooks/packages/usePackageStore";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "./ui/card";
import { Button } from "./ui/button";

type Props = {
  header: string;
  description: string;
  handleClick: () => void;
  loading: boolean;
  error: string | null;
};

function PackageSelector() {
  const { data: packages } = usePackages();
  const { setSelectedPackage, selectedPackage } = usePackageStore();

  return (
    <Select
      onValueChange={(id) => {
        const pkg = packages?.find((p) => p.id === id) ?? null;
        setSelectedPackage(pkg);
      }}
      defaultValue={selectedPackage?.id}
    >
      <SelectTrigger className="w-56">
        <SelectValue placeholder="Select package" />
      </SelectTrigger>
      <SelectContent>
        <SelectGroup>
          {packages?.map((p) => (
            <SelectItem key={p.id} value={p.id}>
              {p.package_id}{" "}
              <span className="text-muted-foreground">@{p.version}</span>
            </SelectItem>
          ))}
        </SelectGroup>
      </SelectContent>
    </Select>
  );
}

export function PackageSelectorCard({
  header,
  description,
  handleClick,
  loading,
  error,
}: Props) {
  const { selectedPackage } = usePackageStore();

  return (
    <>
      <Card className="border-border/60">
        <CardHeader>
          <CardTitle>{header}</CardTitle>
          <CardDescription>{description}</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap items-center gap-3">
            <PackageSelector />
            <Button
              className="w-full sm:w-auto"
              onClick={handleClick}
              disabled={!selectedPackage || loading}
            >
              {loading ? "Analyzing…" : "Analyze"}
            </Button>
          </div>
          {error && <p className="mt-2 text-sm text-red-400">{error}</p>}
        </CardContent>
      </Card>
      {loading && (
        <p className="text-sm text-muted-foreground animate-pulse">
          Working...
        </p>
      )}

      {error && (
        <Card className="border-red-500/40">
          <CardHeader>
            <CardTitle className="text-red-400">Failed</CardTitle>
            <CardDescription>{error}</CardDescription>
          </CardHeader>
        </Card>
      )}
    </>
  );
}

export default PackageSelector;

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { Menu, X } from "lucide-react";
import { useState } from "react";
import { NavLink } from "react-router-dom";

type NavItem = {
  to: string;
  label: string;
  description: string;
};

const navItems: NavItem[] = [
  {
    to: "/scan",
    label: "Dependency Scan",
    description: "OSV dependency audit",
  },
  {
    to: "/packages",
    label: "Packages",
    description: "Manage packages",
  },
];

function linkClassName(isActive: boolean) {
  return cn(
    "group px-3 py-2",
    isActive ? "" : "text-muted-foreground hover:text-foreground",
  );
}

function NavBar() {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  return (
    <>
      <header className="fixed inset-x-0 top-0 z-30 border-b bg-background/90 backdrop-blur md:hidden">
        <div className="flex items-center justify-between px-4 py-3">
          <div>
            <p className="text-sm font-semibold">Dependency Scanner</p>
            <p className="text-xs text-muted-foreground">OSV audit tool</p>
          </div>
          <Button
            variant="outline"
            size="icon"
            onClick={() => setMobileMenuOpen((currentValue) => !currentValue)}
            aria-label={mobileMenuOpen ? "Close navigation" : "Open navigation"}
          >
            {mobileMenuOpen ? <X /> : <Menu />}
          </Button>
        </div>
      </header>

      {mobileMenuOpen && (
        <button
          type="button"
          className="fixed inset-0 z-20 bg-black/40 md:hidden"
          onClick={() => setMobileMenuOpen(false)}
          aria-label="Close navigation overlay"
        />
      )}

      <aside
        className={cn(
          "fixed inset-y-0 left-0 z-40 flex w-44 flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground shadow-xl transition-transform duration-200 md:sticky md:top-0 md:h-screen md:z-auto md:translate-x-0 md:shadow-none",
          mobileMenuOpen ? "translate-x-0" : "-translate-x-full md:translate-x-0",
        )}
      >
        <div className="hidden border-b px-4 py-4 md:block">
          <p className="text-sm font-semibold">Dependency Scanner</p>
          <p className="text-xs text-muted-foreground">OSV audit tool</p>
        </div>

        <nav className="flex-1 space-y-1 overflow-y-auto px-3 py-4 pt-20 md:pt-4">
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              onClick={() => setMobileMenuOpen(false)}
              className={({ isActive }) => linkClassName(isActive)}
            >
              <p className="text-sm font-medium">{item.label}</p>
              <p className="text-xs text-muted-foreground group-hover:text-muted-foreground/90">
                {item.description}
              </p>
            </NavLink>
          ))}
        </nav>
      </aside>
    </>
  );
}

export default NavBar;

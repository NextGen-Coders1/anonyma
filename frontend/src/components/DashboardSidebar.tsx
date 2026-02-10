import { Inbox, Users, Radio, Settings, Shield } from "lucide-react";
import { NavLink } from "@/components/NavLink";

const navItems = [
  { title: "Inbox", url: "/dashboard/inbox", icon: Inbox },
  { title: "Users", url: "/dashboard/users", icon: Users },
  { title: "Broadcasts", url: "/dashboard/broadcasts", icon: Radio },
  { title: "Settings", url: "/dashboard/settings", icon: Settings },
];

const DashboardSidebar = () => {
  return (
    <aside className="flex h-screen w-64 flex-col border-r border-border bg-card/50 backdrop-blur-md">
      {/* Logo */}
      <div className="flex items-center gap-3 border-b border-border px-6 py-5">
        <Shield className="h-6 w-6 text-primary" />
        <span className="font-mono text-lg font-bold gradient-text">ANONYMA</span>
      </div>

      {/* Nav */}
      <nav className="flex-1 space-y-1 p-4">
        {navItems.map((item) => (
          <NavLink
            key={item.url}
            to={item.url}
            end
            className="flex items-center gap-3 rounded-lg px-4 py-3 font-mono text-sm text-muted-foreground transition-all duration-200 hover:bg-muted hover:text-foreground"
            activeClassName="bg-muted text-primary neon-border-purple"
          >
            <item.icon className="h-4 w-4" />
            {item.title}
          </NavLink>
        ))}
      </nav>

      {/* Profile */}
      <div className="border-t border-border p-4">
        <div className="glass flex items-center gap-3 rounded-lg p-3">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary/20 font-mono text-xs font-bold text-primary">
            AG
          </div>
          <div>
            <p className="font-mono text-xs font-semibold text-foreground">Agent_001</p>
            <p className="font-mono text-[10px] text-secondary">● Online</p>
          </div>
        </div>
      </div>
    </aside>
  );
};

export default DashboardSidebar;

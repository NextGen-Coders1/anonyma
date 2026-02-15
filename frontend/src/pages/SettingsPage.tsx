import { Settings, User, Bell, Lock } from "lucide-react";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";

const SettingsPage = () => {
  return (
    <div>
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <Settings className="h-5 w-5 text-primary" />
          <h1 className="font-mono text-2xl font-bold text-foreground">Settings</h1>
        </div>
        <p className="text-sm text-muted-foreground font-mono">Configure your shadow profile</p>
      </div>

      <div className="space-y-6 max-w-xl">
        {[
          { icon: User, title: "Profile", desc: "Update your agent identity" },
          { icon: Bell, title: "Notifications", desc: "Manage alert preferences" },
          { icon: Lock, title: "Security", desc: "Encryption & privacy settings" },
        ].map((item) => (
          <div key={item.title} className="glass flex items-center justify-between rounded-xl p-5">
            <div className="flex items-center gap-4">
              <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
                <item.icon className="h-5 w-5 text-primary" />
              </div>
              <div>
                <h3 className="font-mono text-sm font-semibold text-foreground">{item.title}</h3>
                <p className="font-mono text-xs text-muted-foreground">{item.desc}</p>
              </div>
            </div>
            <Button variant="glass" size="sm" onClick={() => toast.info(`Editing ${item.title}`, { description: "This feature is coming soon." })}>Edit</Button>
          </div>
        ))}
      </div>
    </div>
  );
};

export default SettingsPage;

import { useState } from "react";
import { Settings, User, Bell, Trash2, Save, Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";
import { useAuth } from "@/providers/AuthProvider";
import { auth, preferences } from "@/lib/api";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Switch } from "@/components/ui/switch";

const SettingsPage = () => {
  const { user } = useAuth();
  const [isEditingProfile, setIsEditingProfile] = useState(false);
  const [username, setUsername] = useState(user?.username || "");
  const [bio, setBio] = useState(user?.bio || "");
  const [isSaving, setIsSaving] = useState(false);

  // Notification settings (browser notifications with sound only)
  const [browserNotifications, setBrowserNotifications] = useState(
    localStorage.getItem("browserNotifications") !== "false" // default true
  );
  const [notificationSound, setNotificationSound] = useState(
    localStorage.getItem("notificationSound") !== "false" // default true
  );

  const handleUpdateProfile = async () => {
    setIsSaving(true);
    try {
      await auth.updateProfile({ username, bio });
      toast.success("Profile updated successfully");
      setIsEditingProfile(false);
      window.location.reload();
    } catch (error) {
      toast.error("Failed to update profile");
    } finally {
      setIsSaving(false);
    }
  };

  const handleSaveNotifications = async () => {
    try {
      await preferences.update({
        browser_notifications: browserNotifications,
        notification_sound: notificationSound,
      });
      localStorage.setItem("browserNotifications", browserNotifications.toString());
      localStorage.setItem("notificationSound", notificationSound.toString());
      toast.success("Notification preferences saved");
    } catch (error) {
      console.error("Failed to save preferences:", error);
      toast.error("Failed to save preferences");
    }
  };

  const handleDeleteAccount = async () => {
    if (!window.confirm("Are you sure you want to delete your account? This action is irreversible.")) {
      return;
    }
    try {
      await auth.deleteAccount();
      toast.success("Account deleted");
      window.location.href = "/";
    } catch (error) {
      toast.error("Failed to delete account");
    }
  };

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
        {/* Profile Section */}
        <div className="glass flex flex-col rounded-xl p-5 gap-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
                <User className="h-5 w-5 text-primary" />
              </div>
              <div>
                <h3 className="font-mono text-sm font-semibold text-foreground">Profile</h3>
                <p className="font-mono text-xs text-muted-foreground">Update your agent identity</p>
              </div>
            </div>
            {!isEditingProfile && (
              <Button variant="glass" size="sm" onClick={() => setIsEditingProfile(true)}>Edit</Button>
            )}
          </div>

          {isEditingProfile ? (
            <div className="space-y-4 pt-2">
              <div className="space-y-2">
                <label className="text-xs font-mono text-muted-foreground">Username</label>
                <Input 
                  value={username} 
                  onChange={(e) => setUsername(e.target.value)}
                  className="bg-background/50 font-mono"
                />
              </div>
              <div className="space-y-2">
                <label className="text-xs font-mono text-muted-foreground">Bio</label>
                <Textarea 
                  value={bio} 
                  onChange={(e) => setBio(e.target.value)}
                  className="bg-background/50 font-mono resize-none"
                  placeholder="Describe your shadow self..."
                />
              </div>
              <div className="flex gap-2 justify-end">
                <Button variant="ghost" size="sm" onClick={() => setIsEditingProfile(false)}>Cancel</Button>
                <Button variant="glass" size="sm" onClick={handleUpdateProfile} disabled={isSaving}>
                  <Save className="h-4 w-4 mr-2" />
                  {isSaving ? "Saving..." : "Save Changes"}
                </Button>
              </div>
            </div>
          ) : (
            <div className="pt-2 px-1">
              <div className="flex flex-col gap-1">
                <span className="text-xs font-mono text-muted-foreground uppercase tracking-tighter">Current Username</span>
                <span className="text-sm font-mono text-foreground">{user?.username}</span>
              </div>
              {user?.bio && (
                <div className="flex flex-col gap-1 mt-3">
                  <span className="text-xs font-mono text-muted-foreground uppercase tracking-tighter">Bio</span>
                  <span className="text-sm font-mono text-foreground">{user.bio}</span>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Notifications Section */}
        <div className="glass flex flex-col rounded-xl p-5 gap-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
                <Bell className="h-5 w-5 text-primary" />
              </div>
              <div>
                <h3 className="font-mono text-sm font-semibold text-foreground">Notifications</h3>
                <p className="font-mono text-xs text-muted-foreground">Browser notifications with sound</p>
              </div>
            </div>
          </div>

          <div className="space-y-4 pt-2">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-mono text-foreground">Browser Notifications</p>
                <p className="text-xs font-mono text-muted-foreground">Show desktop notifications</p>
              </div>
              <Switch
                checked={browserNotifications}
                onCheckedChange={setBrowserNotifications}
              />
            </div>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-mono text-foreground">Notification Sound</p>
                <p className="text-xs font-mono text-muted-foreground">Play sound on new messages</p>
              </div>
              <Switch
                checked={notificationSound}
                onCheckedChange={setNotificationSound}
              />
            </div>
            <div className="flex justify-end pt-2">
              <Button variant="glass" size="sm" onClick={handleSaveNotifications}>
                <Check className="h-4 w-4 mr-2" />
                Save Preferences
              </Button>
            </div>
          </div>
        </div>

        {/* Danger Zone */}
        <div className="glass border-destructive/20 flex items-center justify-between rounded-xl p-5 mt-10">
          <div className="flex items-center gap-4">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-destructive/10">
              <Trash2 className="h-5 w-5 text-destructive" />
            </div>
            <div>
              <h3 className="font-mono text-sm font-semibold text-destructive">Danger Zone</h3>
              <p className="font-mono text-xs text-muted-foreground">Permanently delete your account</p>
            </div>
          </div>
          <Button variant="destructive" size="sm" onClick={handleDeleteAccount}>Delete Account</Button>
        </div>
      </div>
    </div>
  );
};

export default SettingsPage;

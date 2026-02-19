import { useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { Shield, Lock, User, Github, Loader2, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { toast } from "sonner";
import { auth } from "@/lib/api";

const LoginPage = () => {
  const [isLogin, setIsLogin] = useState(true);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!username || !password) {
      toast.error("Please fill in all fields");
      return;
    }

    setIsLoading(true);
    try {
      if (isLogin) {
        await auth.login(username, password);
        toast.success("Welcome back, agent.");
      } else {
        await auth.register(username, password);
        toast.success("Identity established.", { description: "You are now logged in." });
      }
      window.location.href = "/dashboard";
    } catch (error: any) {
      console.error("Auth error:", error);
      const message = error.message || (isLogin ? "Login failed" : "Registration failed");
      toast.error(isLogin ? "Login failed" : "Registration failed", { 
        description: message.includes("409") ? "Codename already taken." : 
                     message.includes("400") ? "Invalid input. Password must be 6+ chars." :
                     "Check your credentials or try again." 
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleGithubLogin = () => {
    window.location.href = auth.loginUrl();
  };

  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-background p-4 relative overflow-hidden">
      {/* Background Elements */}
      <div className="absolute inset-0 bg-grid-white/[0.02] bg-[size:50px_50px]" />
      <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 h-[500px] w-[500px] bg-primary/20 blur-[100px] rounded-full opacity-20 pointer-events-none" />

      <div className="z-10 w-full max-w-md space-y-8 glass p-8 rounded-2xl animate-fade-in">
        <div className="flex flex-col items-center text-center">
          <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-primary/20 neon-border-purple">
            <Shield className="h-8 w-8 text-primary" />
          </div>
          <h2 className="text-3xl font-bold tracking-tight gradient-text">ANONYMA</h2>
          <p className="mt-2 text-sm text-muted-foreground font-mono">
            {isLogin ? "Authenticate to access the network" : "Initialize new agent identity"}
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-6">
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="username" className="font-mono text-xs uppercase text-muted-foreground">
                Codename
              </Label>
              <div className="flex gap-2">
                <Input
                  id="username"
                  type="text"
                  placeholder="Enter or generate codename"
                  className="bg-muted/50 font-mono"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  required
                />
                {!isLogin && (
                    <Button 
                      type="button" 
                      variant="outline" 
                      size="icon"
                      onClick={async () => {
                        try {
                            const res = await fetch('https://generate-random.org/api/v1/generate/usernames');
                            const data = await res.json();
                            if (data.data && data.data.length > 0) {
                                setUsername(data.data[0]);
                            }
                        } catch (e) {
                             // Fallback
                             setUsername(`Agent_${Math.floor(Math.random() * 1000)}`);
                        }
                      }}
                      title="Generate Random Codename"
                    >
                      <RefreshCw className="h-4 w-4" />
                    </Button>
                )}
              </div>
            </div>
            <div className="space-y-2">
              <div className="relative">
                <Lock className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
                <Input
                  type="password"
                  placeholder="Passphrase"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="pl-10 font-mono bg-muted/50 border-input focus:neon-border-purple transition-all"
                />
              </div>
            </div>
          </div>

          <Button
            type="submit"
            className="w-full bg-primary hover:bg-primary/90 text-primary-foreground font-bold shadow-[0_0_20px_rgba(124,58,237,0.3)] hover:shadow-[0_0_30px_rgba(124,58,237,0.5)] transition-all duration-300"
            disabled={isLoading}
          >
            {isLoading ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : isLogin ? (
              "Access Network"
            ) : (
              "Initialize Identity"
            )}
          </Button>
        </form>

        <div className="relative">
          <div className="absolute inset-0 flex items-center">
            <span className="w-full border-t border-muted" />
          </div>
          <div className="relative flex justify-center text-xs uppercase">
            <span className="bg-background px-2 text-muted-foreground">Or continue with</span>
          </div>
        </div>

        <Button
          variant="outline"
          className="w-full border-muted bg-background hover:bg-muted transition-colors"
          onClick={handleGithubLogin}
        >
          <Github className="mr-2 h-4 w-4" />
          GitHub
        </Button>

        <div className="text-center text-sm">
          <span className="text-muted-foreground">
            {isLogin ? "New to the network? " : "Already initialized? "}
          </span>
          <button
            onClick={() => setIsLogin(!isLogin)}
            className="font-semibold text-primary hover:underline hover:text-primary/90 transition-colors"
          >
            {isLogin ? "Initialize identity" : "Access network"}
          </button>
        </div>
      </div>
    </div>
  );
};

export default LoginPage;

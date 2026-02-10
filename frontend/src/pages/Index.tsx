import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { Shield, Eye, EyeOff, ArrowRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import ParticleBackground from "@/components/ParticleBackground";
import heroBg from "@/assets/hero-bg.jpg";
import { toast } from "sonner";

const Index = () => {
  const navigate = useNavigate();
  const [isLogin, setIsLogin] = useState(true);
  const [showPassword, setShowPassword] = useState(false);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!username.trim() || !password.trim()) {
      toast.error("Please fill in all fields");
      return;
    }
    toast.success(`Welcome, ${username}`, { description: "Entering the shadows..." });
    navigate("/dashboard/inbox");
  };

  return (
    <div className="relative flex min-h-screen items-center justify-center overflow-hidden">
      {/* Background */}
      <img
        src={heroBg}
        alt=""
        className="absolute inset-0 h-full w-full object-cover opacity-40"
      />
      <ParticleBackground />

      {/* Content */}
      <motion.div
        initial={{ opacity: 0, y: 30 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
        className="glass-strong relative z-10 w-full max-w-md rounded-2xl p-8"
      >
        {/* Header */}
        <div className="mb-8 text-center">
          <motion.div
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            transition={{ delay: 0.2, type: "spring" }}
            className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-primary/10 glow-purple"
          >
            <Shield className="h-8 w-8 text-primary" />
          </motion.div>
          <h1 className="mb-2 font-mono text-3xl font-bold gradient-text">
            Enter the Shadows
          </h1>
          <p className="font-mono text-sm text-muted-foreground">
            {isLogin ? "Welcome back, agent" : "Create your shadow identity"}
          </p>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="mb-1.5 block font-mono text-xs text-muted-foreground uppercase tracking-wider">
              Agent Name
            </label>
            <input
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder="ShadowWalker"
              className="w-full rounded-lg border border-border bg-muted/50 px-4 py-3 font-mono text-sm text-foreground placeholder:text-muted-foreground focus:neon-border-purple focus:outline-none transition-all"
              maxLength={30}
            />
          </div>

          <div>
            <label className="mb-1.5 block font-mono text-xs text-muted-foreground uppercase tracking-wider">
              Passphrase
            </label>
            <div className="relative">
              <input
                type={showPassword ? "text" : "password"}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="••••••••"
                className="w-full rounded-lg border border-border bg-muted/50 px-4 py-3 pr-12 font-mono text-sm text-foreground placeholder:text-muted-foreground focus:neon-border-purple focus:outline-none transition-all"
                maxLength={50}
              />
              <button
                type="button"
                onClick={() => setShowPassword(!showPassword)}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
              >
                {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
              </button>
            </div>
          </div>

          <Button type="submit" variant="hero" size="xl" className="w-full mt-2">
            {isLogin ? "Enter" : "Create Identity"}
            <ArrowRight className="ml-2 h-4 w-4" />
          </Button>
        </form>

        {/* Toggle */}
        <p className="mt-6 text-center font-mono text-xs text-muted-foreground">
          {isLogin ? "No identity yet?" : "Already an agent?"}{" "}
          <button
            onClick={() => setIsLogin(!isLogin)}
            className="text-primary hover:text-primary/80 transition-colors underline underline-offset-4"
          >
            {isLogin ? "Create one" : "Sign in"}
          </button>
        </p>
      </motion.div>
    </div>
  );
};

export default Index;

import { motion } from "framer-motion";
import { Shield, Github } from "lucide-react";
import { Button } from "@/components/ui/button";
import ParticleBackground from "@/components/ParticleBackground";
import heroBg from "@/assets/hero-bg.jpg";
import { auth } from "@/lib/api";

const Index = () => {
  const handleGitHubLogin = () => {
    window.location.href = auth.loginUrl();
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
            Secure anonymous communication platform
          </p>
        </div>

        {/* GitHub Login Button */}
        <Button
          onClick={handleGitHubLogin}
          variant="hero"
          size="xl"
          className="w-full"
        >
          <Github className="mr-2 h-5 w-5" />
          Continue with GitHub
        </Button>

        {/* Info */}
        <p className="mt-6 text-center font-mono text-xs text-muted-foreground">
          Authenticate via GitHub
        </p>
      </motion.div>
    </div>
  );
};

export default Index;

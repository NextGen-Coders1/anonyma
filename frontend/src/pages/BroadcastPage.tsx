import { useState } from "react";
import { motion } from "framer-motion";
import { Radio, Heart, Send } from "lucide-react";
import { Button } from "@/components/ui/button";
import { mockBroadcasts } from "@/lib/mock-data";
import { toast } from "sonner";

const BroadcastPage = () => {
  const [broadcastText, setBroadcastText] = useState("");

  const handleBroadcast = () => {
    if (!broadcastText.trim()) return;
    toast.success("Broadcast sent!", { description: "Your message is now visible to all agents." });
    setBroadcastText("");
  };

  return (
    <div>
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <Radio className="h-5 w-5 text-secondary" />
          <h1 className="font-mono text-2xl font-bold text-foreground">Broadcast Board</h1>
        </div>
        <p className="text-sm text-muted-foreground font-mono">
          Public feed — visible to all agents
        </p>
      </div>

      {/* Broadcast Input */}
      <div className="glass-strong mb-8 rounded-xl p-5">
        <textarea
          value={broadcastText}
          onChange={(e) => setBroadcastText(e.target.value)}
          placeholder="Broadcast to all agents..."
          className="mb-3 h-20 w-full resize-none rounded-lg border border-border bg-muted/50 p-4 font-mono text-sm text-foreground placeholder:text-muted-foreground focus:neon-border-cyan focus:outline-none"
          maxLength={280}
        />
        <div className="flex items-center justify-between">
          <span className="text-xs text-muted-foreground font-mono">{broadcastText.length}/280</span>
          <Button variant="neon-cyan" size="sm" onClick={handleBroadcast} disabled={!broadcastText.trim()}>
            <Send className="mr-2 h-3.5 w-3.5" />
            Broadcast to All
          </Button>
        </div>
      </div>

      {/* Feed */}
      <div className="space-y-4">
        {mockBroadcasts.map((bc, i) => (
          <motion.div
            key={bc.id}
            initial={{ opacity: 0, y: 15 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.05 }}
            className="glass rounded-xl p-5 transition-all duration-300 hover:neon-border-cyan"
          >
            <div className="mb-3 flex items-center gap-2">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-muted font-mono text-xs font-bold text-secondary">
                AN
              </div>
              <span className="font-mono text-xs text-muted-foreground">{bc.timestamp}</span>
            </div>
            <p className="mb-4 font-mono text-sm text-foreground/90 leading-relaxed">{bc.content}</p>
            <div className="flex items-center gap-2 text-muted-foreground">
              <Heart className="h-3.5 w-3.5" />
              <span className="font-mono text-xs">{bc.reactions}</span>
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  );
};

export default BroadcastPage;

import { useState, useEffect, useRef } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { motion } from "framer-motion";
import { Radio, Heart, Send, Loader2, Eye } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { broadcasts, Broadcast } from "@/lib/api";
import { toast } from "sonner";

const BroadcastItem = ({ bc, index }: { bc: Broadcast; index: number }) => {
  const observerRef = useRef<HTMLDivElement>(null);
  const [hasTracked, setHasTracked] = useState(false);

  const viewMutation = useMutation({
    mutationFn: (id: string) => broadcasts.trackView(id),
  });

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && !hasTracked) {
          setHasTracked(true);
          viewMutation.mutate(bc.id);
        }
      },
      { threshold: 0.5 }
    );

    if (observerRef.current) {
      observer.observe(observerRef.current);
    }

    return () => observer.disconnect();
  }, [bc.id, hasTracked, viewMutation]);

  return (
    <motion.div
      ref={observerRef}
      initial={{ opacity: 0, y: 15 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.05 }}
      className="glass rounded-xl p-5 transition-all duration-300 hover:neon-border-cyan"
    >
      <div className="mb-3 flex items-center gap-2">
        <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-muted font-mono text-xs font-bold text-secondary">
          {bc.is_anonymous ? "AN" : (bc.sender_username?.[0] || "?").toUpperCase()}
        </div>
        <div className="flex flex-col">
          <span className="font-mono text-xs font-semibold text-foreground">
            {bc.is_anonymous ? "Anonymous" : bc.sender_username}
          </span>
          <span className="font-mono text-xs text-muted-foreground">
            {new Date(bc.created_at).toLocaleString()}
          </span>
        </div>
      </div>
      <p className="mb-4 font-mono text-sm text-foreground/90 leading-relaxed">{bc.content}</p>
      <div className="flex items-center gap-4 text-muted-foreground">
        <div className="flex items-center gap-1.5">
          <Heart className="h-3.5 w-3.5" />
          <span className="font-mono text-xs">0</span>
        </div>
        <div className="flex items-center gap-1.5">
          <Eye className="h-3.5 w-3.5 text-secondary" />
          <span className="font-mono text-xs text-secondary">{bc.view_count || 0}</span>
        </div>
      </div>
    </motion.div>
  );
};

const BroadcastPage = () => {
  const [broadcastText, setBroadcastText] = useState("");
  const [isAnonymous, setIsAnonymous] = useState(false);
  const queryClient = useQueryClient();

  const { data: broadcastList = [], isLoading } = useQuery({
    queryKey: ['broadcasts'],
    queryFn: broadcasts.list,
    refetchInterval: 5000, // Refetch every 5s to sync view counts
  });

  const createBroadcast = useMutation({
    mutationFn: ({ content, is_anonymous }: { content: string; is_anonymous: boolean }) =>
      broadcasts.create(content, is_anonymous),
    onSuccess: () => {
      toast.success("Broadcast sent!", { description: "Your message is now visible to all agents." });
      setBroadcastText("");
      setIsAnonymous(false);
      queryClient.invalidateQueries({ queryKey: ['broadcasts'] });
    },
    onError: () => {
      toast.error("Failed to send broadcast", { description: "Please try again." });
    },
  });

  const handleBroadcast = () => {
    if (!broadcastText.trim()) return;
    createBroadcast.mutate({ content: broadcastText, is_anonymous: isAnonymous });
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
          <div className="flex items-center gap-4">
            <span className="text-xs text-muted-foreground font-mono">{broadcastText.length}/280</span>
            <div className="flex items-center gap-2">
              <Checkbox id="anonymous" checked={isAnonymous} onCheckedChange={(checked) => setIsAnonymous(!!checked)} />
              <label htmlFor="anonymous" className="text-xs text-muted-foreground font-mono cursor-pointer">
                Post anonymously
              </label>
            </div>
          </div>
          <Button
            variant="neon-cyan"
            size="sm"
            onClick={handleBroadcast}
            disabled={!broadcastText.trim() || createBroadcast.isPending}
          >
            {createBroadcast.isPending ? (
              <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
            ) : (
              <Send className="mr-2 h-3.5 w-3.5" />
            )}
            Broadcast to All
          </Button>
        </div>
      </div>

      {/* Feed */}
      <div className="space-y-4">
        {isLoading && broadcastList.length === 0 ? (
          <div className="flex justify-center py-8">
            <Loader2 className="h-8 w-8 animate-spin text-primary" />
          </div>
        ) : broadcastList.length === 0 ? (
          <div className="glass rounded-xl p-8 text-center">
            <p className="font-mono text-sm text-muted-foreground">No broadcasts yet. Be the first!</p>
          </div>
        ) : (
          broadcastList.map((bc, i) => (
            <BroadcastItem key={bc.id} bc={bc} index={i} />
          ))
        )}
      </div>
    </div>
  );
};

export default BroadcastPage;

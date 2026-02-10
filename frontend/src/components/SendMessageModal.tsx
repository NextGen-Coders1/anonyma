import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { motion, AnimatePresence } from "framer-motion";
import { EyeOff, Send, X, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { messages } from "@/lib/api";
import { toast } from "sonner";

interface SendMessageModalProps {
  isOpen: boolean;
  onClose: () => void;
  recipientId: string;
  recipientUsername: string;
}

const SendMessageModal = ({ isOpen, onClose, recipientId, recipientUsername }: SendMessageModalProps) => {
  const [message, setMessage] = useState("");
  const queryClient = useQueryClient();

  const sendMessage = useMutation({
    mutationFn: (content: string) => messages.send(recipientId, content),
    onSuccess: () => {
      toast.success("Message sent anonymously!", {
        description: `Your secret message was delivered to ${recipientUsername}.`,
      });
      setMessage("");
      onClose();
      queryClient.invalidateQueries({ queryKey: ['inbox'] });
    },
    onError: () => {
      toast.error("Failed to send message", {
        description: "Please try again.",
      });
    },
  });

  const handleSend = () => {
    if (!message.trim()) return;
    sendMessage.mutate(message);
  };

  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="fixed inset-0 z-50 flex items-center justify-center p-4"
          onClick={onClose}
        >
          <div className="absolute inset-0 bg-background/80 backdrop-blur-sm" />
          <motion.div
            initial={{ scale: 0.95, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            exit={{ scale: 0.95, opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="glass-strong relative z-10 w-full max-w-lg rounded-xl p-6"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="mb-6 flex items-center justify-between">
              <div>
                <h3 className="font-mono text-lg font-semibold text-foreground">
                  Send to <span className="text-primary">{recipientUsername}</span>
                </h3>
                <div className="mt-1 flex items-center gap-2 text-sm text-muted-foreground">
                  <EyeOff className="h-3.5 w-3.5 text-secondary" />
                  <span className="font-mono text-secondary">Incognito Mode Active</span>
                </div>
              </div>
              <button
                onClick={onClose}
                className="rounded-lg p-2 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            <textarea
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              placeholder="Type your anonymous message..."
              className="mb-4 h-32 w-full resize-none rounded-lg border border-border bg-muted/50 p-4 font-mono text-sm text-foreground placeholder:text-muted-foreground focus:neon-border-purple focus:outline-none"
              maxLength={500}
            />

            <div className="flex items-center justify-between">
              <span className="text-xs text-muted-foreground font-mono">
                {message.length}/500
              </span>
              <Button
                variant="neon"
                onClick={handleSend}
                disabled={!message.trim() || sendMessage.isPending}
              >
                {sendMessage.isPending ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <Send className="mr-2 h-4 w-4" />
                )}
                Send Secretly
              </Button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};

export default SendMessageModal;

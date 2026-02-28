import { useState, useRef, useEffect } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { motion, AnimatePresence } from "framer-motion";
import { EyeOff, Send, X, Loader2, MessageCircle, Check, XCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { messages, conversations, type Message } from "@/lib/api";
import { toast } from "sonner";
import { MessageActions } from "@/components/MessageActions";
import { TypingIndicator } from "@/components/TypingIndicator";
import { EmojiPickerComponent } from "@/components/EmojiPickerComponent";
import { useDrafts } from "@/hooks/useDrafts";
import { useTypingIndicator } from "@/hooks/useTypingIndicator";

interface ConversationViewProps {
    /** The message ID to reply to (used to determine thread) */
    messageId: string;
    threadId: string;
    /** Recipient's name — only provided when the current user is the sender */
    toUsername?: string;
    isOpen: boolean;
    onClose: () => void;
}

const ConversationView = ({ messageId, threadId, toUsername, isOpen, onClose }: ConversationViewProps) => {
    const [reply, setReply] = useState("");
    const [hoveredMessageId, setHoveredMessageId] = useState<string | null>(null);
    const [editingMessageId, setEditingMessageId] = useState<string | null>(null);
    const [editContent, setEditContent] = useState("");
    const [isTyping, setIsTyping] = useState(false);
    const [typingUsername, setTypingUsername] = useState<string | undefined>(undefined);
    const queryClient = useQueryClient();
    const bottomRef = useRef<HTMLDivElement>(null);
    
    // Drafts
    const { draft, saveDraft, clearDraft } = useDrafts(threadId);
    
    // Typing indicator
    const { handleTyping } = useTypingIndicator(threadId);

    // Load draft on mount
    useEffect(() => {
        if (draft && !reply) {
            setReply(draft);
        }
    }, [draft, reply]);

    const { data: thread = [], isLoading } = useQuery({
        queryKey: ["thread", threadId],
        queryFn: () => conversations.getThread(threadId),
        enabled: isOpen && !!threadId,
        refetchInterval: 2000, // Poll every 2 seconds as backup to SSE
    });

    // Auto-scroll to bottom when messages arrive
    useEffect(() => {
        if (thread.length > 0) {
            bottomRef.current?.scrollIntoView({ behavior: "smooth" });
        }
    }, [thread.length]);

    // Pull fresh thread when conversation is opened (also marks server-side as read)
    useEffect(() => {
        if (isOpen) {
            queryClient.invalidateQueries({ queryKey: ["thread", threadId] });
            // Mark as read in list view too
            queryClient.invalidateQueries({ queryKey: ["conversations"] });
            queryClient.invalidateQueries({ queryKey: ["inbox"] });
        }
    }, [isOpen, threadId, queryClient]);

    const replyMutation = useMutation({
        mutationFn: (content: string) => messages.reply(messageId, content),
        onSuccess: () => {
            setReply("");
            clearDraft();
            queryClient.invalidateQueries({ queryKey: ["thread", threadId] });
            queryClient.invalidateQueries({ queryKey: ["inbox"] });
            queryClient.invalidateQueries({ queryKey: ["conversations"] });
        },
        onError: () => {
            toast.error("Failed to send reply");
        },
    });

    const editMutation = useMutation({
        mutationFn: ({ msgId, content }: { msgId: string; content: string }) =>
            messages.edit(msgId, content),
        onSuccess: () => {
            setEditingMessageId(null);
            setEditContent("");
            queryClient.invalidateQueries({ queryKey: ["thread", threadId] });
            toast.success("Message edited");
        },
        onError: () => {
            toast.error("Failed to edit message");
        },
    });

    const deleteMutation = useMutation({
        mutationFn: (msgId: string) => messages.delete(msgId),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["thread", threadId] });
            queryClient.invalidateQueries({ queryKey: ["inbox"] });
            queryClient.invalidateQueries({ queryKey: ["conversations"] });
            toast.success("Message deleted");
        },
        onError: () => {
            toast.error("Failed to delete message");
        },
    });

    const pinMutation = useMutation({
        mutationFn: (msgId: string) => messages.pin(msgId),
        onSuccess: (data) => {
            queryClient.invalidateQueries({ queryKey: ["thread", threadId] });
            toast.success(data.pinned ? "Message pinned" : "Message unpinned");
        },
        onError: () => {
            toast.error("Failed to pin message");
        },
    });

    const reactMutation = useMutation({
        mutationFn: ({ msgId, emoji }: { msgId: string; emoji: string }) =>
            messages.react(msgId, emoji),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["thread", threadId] });
            queryClient.invalidateQueries({ queryKey: ["inbox"] });
            queryClient.invalidateQueries({ queryKey: ["conversations"] });
        },
        onError: () => {
            toast.error("Failed to add reaction");
        },
    });

    const handleSend = () => {
        if (!reply.trim() || replyMutation.isPending) return;
        replyMutation.mutate(reply);
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            handleSend();
        }
    };

    const handleReplyChange = (value: string) => {
        setReply(value);
        saveDraft(value);
        handleTyping();
    };

    const handleEdit = (msgId: string, content: string) => {
        setEditingMessageId(msgId);
        setEditContent(content);
    };

    const handleSaveEdit = () => {
        if (!editContent.trim() || !editingMessageId) return;
        editMutation.mutate({ msgId: editingMessageId, content: editContent });
    };

    const handleCancelEdit = () => {
        setEditingMessageId(null);
        setEditContent("");
    };

    // Listen for typing events via SSE
    useEffect(() => {
        const handleTypingEvent = (event: CustomEvent) => {
            try {
                const data = event.detail;
                // Only show typing indicator if it's for this thread and not from current user
                if (data.thread_id === threadId && !data.is_current_user) {
                    setIsTyping(true);
                    // Use the username from the event, or fall back to toUsername
                    setTypingUsername(data.username || toUsername);
                    // Clear typing indicator after 5 seconds
                    const timeout = setTimeout(() => {
                        setIsTyping(false);
                        setTypingUsername(undefined);
                    }, 5000);
                    return () => clearTimeout(timeout);
                }
            } catch (error) {
                console.error('Failed to handle typing event:', error);
            }
        };

        window.addEventListener('typing-indicator', handleTypingEvent as EventListener);
        
        return () => {
            window.removeEventListener('typing-indicator', handleTypingEvent as EventListener);
        };
    }, [threadId, toUsername]);

    // Header label: sender sees recipient's name, recipient sees "Anonymous Agent"
    const chatTitle = toUsername ? `Chat with ${toUsername}` : "Anonymous Agent";

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
                        initial={{ scale: 0.95, opacity: 0, y: 20 }}
                        animate={{ scale: 1, opacity: 1, y: 0 }}
                        exit={{ scale: 0.95, opacity: 0, y: 20 }}
                        transition={{ duration: 0.2 }}
                        className="glass-strong relative z-10 flex w-full max-w-lg flex-col rounded-xl"
                        style={{ height: "min(600px, 85vh)" }}
                        onClick={(e) => e.stopPropagation()}
                    >
                        {/* Header */}
                        <div className="flex items-center justify-between border-b border-border p-4">
                            <div className="flex items-center gap-2">
                                <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary/10">
                                    <MessageCircle className="h-4 w-4 text-primary" />
                                </div>
                                <div>
                                    <h3 className="font-mono text-sm font-semibold text-foreground">{chatTitle}</h3>
                                    <p className="flex items-center gap-1 text-xs text-muted-foreground font-mono">
                                        <EyeOff className="h-3 w-3 text-secondary" />
                                        <span className="text-secondary">Anonymous thread</span>
                                    </p>
                                </div>
                            </div>
                            <button
                                onClick={onClose}
                                className="rounded-lg p-2 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
                            >
                                <X className="h-4 w-4" />
                            </button>
                        </div>

                        {/* Messages */}
                        <div className="flex-1 overflow-y-auto p-4 space-y-3">
                            {isLoading ? (
                                <div className="flex justify-center py-8">
                                    <Loader2 className="h-6 w-6 animate-spin text-primary" />
                                </div>
                            ) : thread.length === 0 ? (
                                <p className="text-center font-mono text-sm text-muted-foreground">No messages yet.</p>
                            ) : (
                                thread.map((msg: Message) => (
                                    <motion.div
                                        key={msg.id}
                                        initial={{ opacity: 0, y: 8 }}
                                        animate={{ opacity: 1, y: 0 }}
                                        className={`flex flex-col group ${msg.is_mine ? "items-end" : "items-start"}`}
                                        onMouseEnter={() => setHoveredMessageId(msg.id)}
                                        onMouseLeave={() => setHoveredMessageId(null)}
                                    >
                                        {editingMessageId === msg.id ? (
                                            // Edit mode
                                            <div className="w-full max-w-[75%] space-y-2">
                                                <textarea
                                                    value={editContent}
                                                    onChange={(e) => setEditContent(e.target.value)}
                                                    className="w-full resize-none rounded-lg border border-border bg-muted/50 p-3 font-mono text-sm text-foreground focus:neon-border-purple focus:outline-none"
                                                    rows={3}
                                                    autoFocus
                                                />
                                                <div className="flex gap-2 justify-end">
                                                    <Button
                                                        variant="ghost"
                                                        size="sm"
                                                        onClick={handleCancelEdit}
                                                    >
                                                        <XCircle className="h-4 w-4 mr-1" />
                                                        Cancel
                                                    </Button>
                                                    <Button
                                                        variant="default"
                                                        size="sm"
                                                        onClick={handleSaveEdit}
                                                        disabled={!editContent.trim()}
                                                    >
                                                        <Check className="h-4 w-4 mr-1" />
                                                        Save
                                                    </Button>
                                                </div>
                                            </div>
                                        ) : (
                                            // Normal message display
                                            <div className="flex items-start gap-2 max-w-[75%]">
                                                <div
                                                    className={`flex-1 rounded-2xl px-4 py-2.5 font-mono text-sm shadow-sm ${msg.is_mine
                                                            ? "bg-primary/20 text-foreground rounded-br-sm border border-primary/30"
                                                            : "bg-muted/80 text-foreground rounded-bl-sm border border-border"
                                                        }`}
                                                >
                                                    {/* Sender label */}
                                                    <p className={`mb-1 text-[10px] font-semibold ${msg.is_mine ? "text-primary text-right" : "text-secondary"}`}>
                                                        {msg.is_mine ? "You" : (toUsername || "Anonymous Agent")}
                                                    </p>
                                                    <p className="leading-relaxed">{msg.content}</p>
                                                    
                                                    {/* Edited indicator */}
                                                    {msg.edited_at && (
                                                        <p className="mt-1 text-[9px] text-muted-foreground italic">
                                                            (edited)
                                                        </p>
                                                    )}
                                                    
                                                    <div className="flex items-center justify-between mt-1">
                                                        <p className="text-[10px] text-muted-foreground">
                                                            {new Date(msg.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                                                        </p>
                                                        
                                                        {/* Read receipt for sender */}
                                                        {msg.is_mine && msg.read_at && (
                                                            <p className="text-[10px] text-primary flex items-center gap-1">
                                                                <Check className="h-3 w-3" />
                                                                Read
                                                            </p>
                                                        )}
                                                    </div>
                                                    
                                                    {/* Individual message reactions */}
                                                    {msg.reactions && Object.keys(msg.reactions).length > 0 && (
                                                        <div className="mt-2 flex flex-wrap gap-1">
                                                            {Object.entries(msg.reactions).map(([emoji, count]) => (
                                                                <div key={emoji} className="flex items-center gap-0.5 rounded-full bg-background/50 px-1.5 py-0.5 text-[10px]">
                                                                    <span>{emoji}</span>
                                                                    <span className="font-bold text-primary">{count as number}</span>
                                                                </div>
                                                            ))}
                                                        </div>
                                                    )}
                                                </div>
                                                
                                                {/* Message actions */}
                                                <MessageActions
                                                    messageId={msg.id}
                                                    content={msg.content}
                                                    isMine={msg.is_mine}
                                                    onEdit={msg.is_mine ? handleEdit : undefined}
                                                    onDelete={msg.is_mine ? (id) => deleteMutation.mutate(id) : undefined}
                                                    onPin={(id) => pinMutation.mutate(id)}
                                                />
                                            </div>
                                        )}
                                        
                                        {/* Reaction picker on hover */}
                                        <AnimatePresence>
                                            {hoveredMessageId === msg.id && editingMessageId !== msg.id && (
                                                <motion.div
                                                    initial={{ opacity: 0, scale: 0.9 }}
                                                    animate={{ opacity: 1, scale: 1 }}
                                                    exit={{ opacity: 0, scale: 0.9 }}
                                                    className="mt-1"
                                                >
                                                    <EmojiPickerComponent
                                                        onEmojiSelect={(emoji) => reactMutation.mutate({ msgId: msg.id, emoji })}
                                                        size="sm"
                                                    />
                                                </motion.div>
                                            )}
                                        </AnimatePresence>
                                    </motion.div>
                                ))
                            )}
                            
                            {/* Typing indicator */}
                            <AnimatePresence>
                                {isTyping && <TypingIndicator username={typingUsername || toUsername} />}
                            </AnimatePresence>
                            
                            <div ref={bottomRef} />
                        </div>

                        {/* Reply box */}
                        <div className="border-t border-border p-4">
                            <div className="flex gap-2">
                                <textarea
                                    value={reply}
                                    onChange={(e) => handleReplyChange(e.target.value)}
                                    onKeyDown={handleKeyDown}
                                    placeholder="Reply anonymously… (Enter to send)"
                                    rows={2}
                                    className="flex-1 resize-none rounded-lg border border-border bg-muted/50 p-3 font-mono text-sm text-foreground placeholder:text-muted-foreground focus:neon-border-purple focus:outline-none"
                                    maxLength={500}
                                />
                                <div className="flex flex-col gap-2">
                                    <EmojiPickerComponent
                                        onEmojiSelect={(emoji) => handleReplyChange(reply + emoji)}
                                        size="sm"
                                    />
                                    <Button
                                        variant="neon"
                                        size="sm"
                                        onClick={handleSend}
                                        disabled={!reply.trim() || replyMutation.isPending}
                                    >
                                        {replyMutation.isPending ? (
                                            <Loader2 className="h-4 w-4 animate-spin" />
                                        ) : (
                                            <Send className="h-4 w-4" />
                                        )}
                                    </Button>
                                </div>
                            </div>
                            {draft && (
                                <p className="mt-2 text-[10px] text-muted-foreground font-mono">
                                    Draft saved
                                </p>
                            )}
                        </div>
                    </motion.div>
                </motion.div>
            )}
        </AnimatePresence>
    );
};

export default ConversationView;

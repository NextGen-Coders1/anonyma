import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { MessageCircle, Send, Loader2, Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { broadcastComments, type BroadcastComment } from '@/lib/api';
import { toast } from 'sonner';
import { motion, AnimatePresence } from 'framer-motion';
import { EmojiPickerComponent } from '@/components/EmojiPickerComponent';
import { useAuth } from '@/providers/AuthProvider';

interface BroadcastCommentsProps {
  broadcastId: string;
}

export function BroadcastComments({ broadcastId }: BroadcastCommentsProps) {
  const [comment, setComment] = useState('');
  const [replyTo, setReplyTo] = useState<string | null>(null);
  const queryClient = useQueryClient();
  const { user } = useAuth();

  const { data: comments = [], isLoading } = useQuery({
    queryKey: ['broadcast-comments', broadcastId],
    queryFn: () => broadcastComments.list(broadcastId),
  });

  const createMutation = useMutation({
    mutationFn: (content: string) =>
      broadcastComments.create(broadcastId, content, replyTo || undefined),
    onSuccess: () => {
      setComment('');
      setReplyTo(null);
      queryClient.invalidateQueries({ queryKey: ['broadcast-comments', broadcastId] });
      toast.success('Comment posted');
    },
    onError: () => {
      toast.error('Failed to post comment');
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (commentId: string) => broadcastComments.delete(commentId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['broadcast-comments', broadcastId] });
      toast.success('Comment deleted');
    },
    onError: () => {
      toast.error('Failed to delete comment');
    },
  });

  const reactMutation = useMutation({
    mutationFn: ({ commentId, emoji }: { commentId: string; emoji: string }) =>
      broadcastComments.react(commentId, emoji),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['broadcast-comments', broadcastId] });
    },
    onError: () => {
      toast.error('Failed to add reaction');
    },
  });

  const handleSubmit = () => {
    if (!comment.trim()) return;
    createMutation.mutate(comment);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  // Organize comments into threads
  const topLevelComments = comments.filter((c) => !c.parent_comment_id);
  const getReplies = (parentId: string) =>
    comments.filter((c) => c.parent_comment_id === parentId);

  const CommentItem = ({ comment: c, isReply = false }: { comment: BroadcastComment; isReply?: boolean }) => (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className={`${isReply ? 'ml-8 mt-2' : 'mt-3'} glass rounded-lg p-3`}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <span className="font-mono text-xs font-semibold text-primary">
              {c.username || 'Anonymous'}
            </span>
            <span className="text-[10px] text-muted-foreground font-mono">
              {new Date(c.created_at).toLocaleString()}
            </span>
          </div>
          <p className="font-mono text-sm text-foreground">{c.content}</p>

          {/* Reactions */}
          {c.reactions && Object.keys(c.reactions).length > 0 && (
            <div className="mt-2 flex flex-wrap gap-1">
              {Object.entries(c.reactions).map(([emoji, count]) => (
                <div
                  key={emoji}
                  className="flex items-center gap-0.5 rounded-full bg-muted px-2 py-0.5 text-xs"
                >
                  <span>{emoji}</span>
                  <span className="font-bold text-primary">{count as number}</span>
                </div>
              ))}
            </div>
          )}

          {/* Actions */}
          <div className="mt-2 flex items-center gap-2">
            <EmojiPickerComponent
              onEmojiSelect={(emoji) => reactMutation.mutate({ commentId: c.id, emoji })}
              size="sm"
            />
            {!isReply && (
              <Button
                variant="ghost"
                size="sm"
                className="h-6 text-xs"
                onClick={() => setReplyTo(c.id)}
              >
                Reply
              </Button>
            )}
            {user?.id === c.user_id && (
              <Button
                variant="ghost"
                size="sm"
                className="h-6 text-xs text-destructive"
                onClick={() => {
                  if (window.confirm('Delete this comment?')) {
                    deleteMutation.mutate(c.id);
                  }
                }}
              >
                <Trash2 className="h-3 w-3" />
              </Button>
            )}
          </div>
        </div>
      </div>

      {/* Replies */}
      {!isReply && getReplies(c.id).map((reply) => (
        <CommentItem key={reply.id} comment={reply} isReply />
      ))}
    </motion.div>
  );

  return (
    <div className="mt-6 border-t border-border pt-6">
      <div className="flex items-center gap-2 mb-4">
        <MessageCircle className="h-4 w-4 text-primary" />
        <h3 className="font-mono text-sm font-semibold text-foreground">
          Comments ({comments.length})
        </h3>
      </div>

      {/* Comment input */}
      <div className="mb-4">
        {replyTo && (
          <div className="mb-2 flex items-center gap-2 text-xs text-muted-foreground font-mono">
            <span>Replying to comment</span>
            <Button
              variant="ghost"
              size="sm"
              className="h-5 text-xs"
              onClick={() => setReplyTo(null)}
            >
              Cancel
            </Button>
          </div>
        )}
        <div className="flex gap-2">
          <textarea
            value={comment}
            onChange={(e) => setComment(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Add a comment..."
            rows={2}
            className="flex-1 resize-none rounded-lg border border-border bg-muted/50 p-3 font-mono text-sm text-foreground placeholder:text-muted-foreground focus:neon-border-purple focus:outline-none"
            maxLength={500}
          />
          <div className="flex flex-col gap-2">
            <EmojiPickerComponent
              onEmojiSelect={(emoji) => setComment(comment + emoji)}
              size="sm"
            />
            <Button
              variant="neon"
              size="sm"
              onClick={handleSubmit}
              disabled={!comment.trim() || createMutation.isPending}
            >
              {createMutation.isPending ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Send className="h-4 w-4" />
              )}
            </Button>
          </div>
        </div>
      </div>

      {/* Comments list */}
      {isLoading ? (
        <div className="flex justify-center py-8">
          <Loader2 className="h-6 w-6 animate-spin text-primary" />
        </div>
      ) : comments.length === 0 ? (
        <p className="text-center font-mono text-sm text-muted-foreground py-8">
          No comments yet. Be the first to comment!
        </p>
      ) : (
        <AnimatePresence>
          {topLevelComments.map((c) => (
            <CommentItem key={c.id} comment={c} />
          ))}
        </AnimatePresence>
      )}
    </div>
  );
}

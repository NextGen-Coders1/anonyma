import { motion } from 'framer-motion';

interface TypingIndicatorProps {
  username?: string;
}

export function TypingIndicator({ username }: TypingIndicatorProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: 10 }}
      className="flex items-center gap-2 px-4 py-2"
    >
      <div className="flex gap-1">
        <motion.div
          className="h-2 w-2 rounded-full bg-primary"
          animate={{ scale: [1, 1.2, 1] }}
          transition={{ duration: 1, repeat: Infinity, delay: 0 }}
        />
        <motion.div
          className="h-2 w-2 rounded-full bg-primary"
          animate={{ scale: [1, 1.2, 1] }}
          transition={{ duration: 1, repeat: Infinity, delay: 0.2 }}
        />
        <motion.div
          className="h-2 w-2 rounded-full bg-primary"
          animate={{ scale: [1, 1.2, 1] }}
          transition={{ duration: 1, repeat: Infinity, delay: 0.4 }}
        />
      </div>
      <span className="text-xs text-muted-foreground font-mono">
        {username || 'Someone'} is typing...
      </span>
    </motion.div>
  );
}

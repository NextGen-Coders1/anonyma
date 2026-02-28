import { useEffect, useRef, useCallback } from 'react';
import { conversations } from '@/lib/api';

export function useTypingIndicator(threadId: string, enabled: boolean = true) {
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);
  const lastSentRef = useRef<number>(0);

  const sendTypingIndicator = useCallback(() => {
    if (!enabled || !threadId) return;

    const now = Date.now();
    // Only send if last sent was more than 3 seconds ago
    if (now - lastSentRef.current < 3000) return;

    lastSentRef.current = now;
    conversations.sendTyping(threadId).catch(err => {
      console.error('Failed to send typing indicator:', err);
    });
  }, [threadId, enabled]);

  const handleTyping = useCallback(() => {
    sendTypingIndicator();

    // Clear existing timeout
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }

    // Set new timeout to stop typing after 3 seconds
    timeoutRef.current = setTimeout(() => {
      // Typing stopped
    }, 3000);
  }, [sendTypingIndicator]);

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  return { handleTyping };
}

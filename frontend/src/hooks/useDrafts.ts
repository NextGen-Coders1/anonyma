import { useState, useEffect, useCallback } from 'react';

const DRAFT_PREFIX = 'draft_';

export function useDrafts(threadId: string) {
  const [draft, setDraft] = useState('');

  // Load draft on mount
  useEffect(() => {
    const key = `${DRAFT_PREFIX}${threadId}`;
    const saved = localStorage.getItem(key);
    if (saved) {
      setDraft(saved);
    }
  }, [threadId]);

  // Save draft to localStorage
  const saveDraft = useCallback((content: string) => {
    setDraft(content);
    const key = `${DRAFT_PREFIX}${threadId}`;
    if (content.trim()) {
      localStorage.setItem(key, content);
    } else {
      localStorage.removeItem(key);
    }
  }, [threadId]);

  // Clear draft
  const clearDraft = useCallback(() => {
    setDraft('');
    const key = `${DRAFT_PREFIX}${threadId}`;
    localStorage.removeItem(key);
  }, [threadId]);

  return { draft, saveDraft, clearDraft };
}

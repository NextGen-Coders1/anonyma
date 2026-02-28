// API Types
export interface User {
  id: string;
  username: string;
  provider: string;
  bio?: string;
  avatar_url?: string;
  created_at: string;
}

export interface Message {
  id: string;
  thread_id: string;
  content: string;
  is_mine: boolean;
  created_at: string;
  is_read: boolean;
  reactions?: Record<string, number>;
  /** Number of unread messages in this thread (only in conversation list) */
  unread_count?: number;
  /** Recipient's username — only populated if the current user is the sender */
  to_username?: string;
  /** Timestamp when message was edited */
  edited_at?: string;
  /** Timestamp when message was read */
  read_at?: string;
  /** Whether message is deleted */
  deleted_at?: string;
}

export interface BroadcastComment {
  id: string;
  broadcast_id: string;
  user_id: string;
  username: string | null;
  content: string;
  parent_comment_id: string | null;
  created_at: string;
  reactions?: Record<string, number>;
}

export interface UserPreferences {
  theme: string;
  notification_sound: boolean;
  browser_notifications: boolean;
  show_read_receipts: boolean;
  show_typing_indicators: boolean;
}

export interface Broadcast {
  id: string;
  sender_username: string | null;
  content: string;
  is_anonymous: boolean;
  created_at: string;
  view_count: number;
}

// API Client
export const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

async function apiRequest<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${API_URL}${endpoint}`, {
    ...options,
    credentials: 'include', // Include cookies for session
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!response.ok) {
    throw new Error(`API Error: ${response.status} ${response.statusText}`);
  }

  // Handle empty responses (like 201 Created)
  const contentLength = response.headers.get("Content-Length");
  if (contentLength === "0" || response.status === 204) {
    return {} as T;
  }

  // Sometimes 201 has no content but no Content-Length header, check text
  const text = await response.text();
  return text ? JSON.parse(text) : ({} as T);
}

// Auth API
export const auth = {
  getMe: () => apiRequest<User>('/api/me'),
  updateProfile: (data: { username?: string; bio?: string; avatar_url?: string }) =>
    apiRequest<User>('/api/me', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  deleteAccount: () =>
    apiRequest<void>('/api/me', {
      method: 'DELETE',
    }),
  login: (username: string, password: string) => apiRequest<void>('/auth/login', {
    method: 'POST',
    body: JSON.stringify({ username, password }),
  }),
  register: (username: string, password: string) => apiRequest<void>('/auth/register', {
    method: 'POST',
    body: JSON.stringify({ username, password }),
  }),
  loginUrl: () => `${API_URL}/auth/github?success_url=${encodeURIComponent('http://localhost:8080/dashboard')}`,
  logoutUrl: () => `${API_URL}/logout`,
};

// Users API
export const users = {
  list: () => apiRequest<User[]>('/api/users'),
  block: (userId: string) =>
    apiRequest<void>(`/api/users/${userId}/block`, {
      method: 'POST',
    }),
  unblock: (userId: string) =>
    apiRequest<void>(`/api/users/${userId}/unblock`, {
      method: 'POST',
    }),
  getBlocked: () => apiRequest<string[]>('/api/users/blocked'),
};

// Messages API
export const messages = {
  inbox: () => apiRequest<Message[]>('/api/messages/inbox'),
  search: (query: string, limit = 50) =>
    apiRequest<Message[]>(`/api/messages/search?q=${encodeURIComponent(query)}&limit=${limit}`),
  send: (recipientId: string, content: string) =>
    apiRequest<void>('/api/messages', {
      method: 'POST',
      body: JSON.stringify({ recipient_id: recipientId, content }),
    }),
  reply: (messageId: string, content: string) =>
    apiRequest<void>(`/api/messages/${messageId}/reply`, {
      method: 'POST',
      body: JSON.stringify({ content }),
    }),
  edit: (messageId: string, content: string) =>
    apiRequest<void>(`/api/messages/${messageId}/edit`, {
      method: 'POST',
      body: JSON.stringify({ content }),
    }),
  delete: (messageId: string) =>
    apiRequest<void>(`/api/messages/${messageId}/delete`, {
      method: 'DELETE',
    }),
  pin: (messageId: string) =>
    apiRequest<{ pinned: boolean }>(`/api/messages/${messageId}/pin`, {
      method: 'POST',
    }),
  react: (messageId: string, emoji: string) =>
    apiRequest<void>(`/api/messages/${messageId}/react`, {
      method: 'POST',
      body: JSON.stringify({ emoji }),
    }),
};

// Conversations API
export const conversations = {
  list: () => apiRequest<Message[]>('/api/conversations'),
  getThread: (threadId: string) => apiRequest<Message[]>(`/api/conversations/${threadId}`),
  deleteThread: (threadId: string) =>
    apiRequest<void>(`/api/conversations/${threadId}/delete`, {
      method: 'DELETE',
    }),
  pinThread: (threadId: string) =>
    apiRequest<{ pinned: boolean }>(`/api/conversations/${threadId}/pin`, {
      method: 'POST',
    }),
  sendTyping: (threadId: string) =>
    apiRequest<void>(`/api/conversations/${threadId}/typing`, {
      method: 'POST',
    }),
};

// Broadcasts API
export const broadcasts = {
  list: () => apiRequest<Broadcast[]>('/api/broadcasts'),
  create: (content: string, isAnonymous: boolean) =>
    apiRequest<void>('/api/broadcasts', {
      method: 'POST',
      body: JSON.stringify({ content, is_anonymous: isAnonymous }),
    }),
  trackView: (broadcastId: string) =>
    apiRequest<void>(`/api/broadcasts/${broadcastId}/view`, {
      method: 'POST',
    }),
};

// Broadcast Comments API
export const broadcastComments = {
  list: (broadcastId: string) =>
    apiRequest<BroadcastComment[]>(`/api/broadcasts/${broadcastId}/comments`),
  create: (broadcastId: string, content: string, parentCommentId?: string) =>
    apiRequest<void>(`/api/broadcasts/${broadcastId}/comments`, {
      method: 'POST',
      body: JSON.stringify({ content, parent_comment_id: parentCommentId }),
    }),
  react: (commentId: string, emoji: string) =>
    apiRequest<void>(`/api/broadcasts/comments/${commentId}/react`, {
      method: 'POST',
      body: JSON.stringify({ emoji }),
    }),
  delete: (commentId: string) =>
    apiRequest<void>(`/api/broadcasts/comments/${commentId}/delete`, {
      method: 'DELETE',
    }),
};

// User Preferences API
export const preferences = {
  get: () => apiRequest<UserPreferences>('/api/preferences'),
  update: (prefs: Partial<UserPreferences>) =>
    apiRequest<void>('/api/preferences', {
      method: 'POST',
      body: JSON.stringify(prefs),
    }),
};

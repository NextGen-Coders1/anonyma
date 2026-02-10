// API Types
export interface User {
  id: string;
  username: string;
  provider: string;
  created_at: string;
}

export interface Message {
  id: string;
  content: string;
  created_at: string;
  is_read: boolean;
}

export interface Broadcast {
  id: string;
  sender_username: string | null;
  content: string;
  is_anonymous: boolean;
  created_at: string;
}

// API Client
const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

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

  return response.json();
}

// Auth API
export const auth = {
  getMe: () => apiRequest<User>('/api/me'),
  loginUrl: () => `${API_URL}/auth/github`,
  logoutUrl: () => `${API_URL}/logout`,
};

// Users API
export const users = {
  list: () => apiRequest<User[]>('/api/users'),
};

// Messages API
export const messages = {
  inbox: () => apiRequest<Message[]>('/api/messages/inbox'),
  send: (recipientId: string, content: string) =>
    apiRequest<void>('/api/messages', {
      method: 'POST',
      body: JSON.stringify({ recipient_id: recipientId, content }),
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
};

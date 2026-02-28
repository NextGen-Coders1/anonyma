import { useEffect, useRef } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { API_URL } from "@/lib/api";
import { useNotificationSound } from "@/hooks/useNotificationSound";

/**
 * Opens a persistent SSE connection to /api/events.
 * On receiving events, invalidates the relevant TanStack Query caches so UI
 * updates in real-time without any manual refresh.
 *
 * Mount this once at the DashboardLayout level.
 */
export function useRealtimeEvents() {
    const queryClient = useQueryClient();
    const esRef = useRef<EventSource | null>(null);
    const { play: playNotificationSound } = useNotificationSound();
    const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);

    useEffect(() => {
        function connect() {
            // Clear any existing reconnect timeout
            if (reconnectTimeoutRef.current) {
                clearTimeout(reconnectTimeoutRef.current);
                reconnectTimeoutRef.current = null;
            }

            // EventSource with credentials (cookies) — supported by all modern browsers
            const es = new EventSource(`${API_URL}/api/events`, { withCredentials: true });
            esRef.current = es;

            es.onopen = () => {
                console.log('SSE connection established');
            };

            es.addEventListener("new_message", (event) => {
                console.log('New message event received:', event.data);
                
                // Refresh both the flat inbox and the conversations/thread list
                queryClient.invalidateQueries({ queryKey: ["inbox"] });
                queryClient.invalidateQueries({ queryKey: ["conversations"] });
                
                // Also invalidate the specific thread if we have the thread_id
                try {
                    const data = JSON.parse(event.data);
                    if (data.thread_id) {
                        queryClient.invalidateQueries({ queryKey: ["thread", data.thread_id] });
                    }
                    
                    // Play notification sound
                    playNotificationSound();
                    
                    // Show browser notification if enabled
                    if (Notification.permission === "granted") {
                        const notification = new Notification("New Message", {
                            body: "You have received a new anonymous message",
                            icon: "/favicon.ico",
                            tag: "new-message",
                            requireInteraction: false,
                        });
                        
                        // Handle notification click - focus window and navigate to inbox
                        notification.onclick = () => {
                            window.focus();
                            window.location.href = '/dashboard/inbox';
                            notification.close();
                        };
                    }
                } catch (e) {
                    console.error('Failed to parse message event data:', e);
                }
            });

            es.addEventListener("message_reaction", (event) => {
                console.log('Message reaction event received:', event.data);
                try {
                    const data = JSON.parse(event.data);
                    if (data.thread_id) {
                        queryClient.invalidateQueries({ queryKey: ["thread", data.thread_id] });
                    }
                    queryClient.invalidateQueries({ queryKey: ["conversations"] });
                } catch (e) {
                    console.error('Failed to parse reaction event data:', e);
                }
            });

            es.addEventListener("typing", (event) => {
                console.log('Typing event received:', event.data);
                // Typing events are handled by individual conversation components
                // We dispatch a custom event that components can listen to
                try {
                    const data = JSON.parse(event.data);
                    // Add a flag to indicate this is not from the current user
                    data.is_current_user = false;
                    window.dispatchEvent(new CustomEvent('typing-indicator', { detail: data }));
                } catch (e) {
                    console.error('Failed to parse typing event data:', e);
                }
            });

            es.addEventListener("read_receipt", (event) => {
                console.log('Read receipt event received:', event.data);
                try {
                    const data = JSON.parse(event.data);
                    if (data.thread_id) {
                        queryClient.invalidateQueries({ queryKey: ["thread", data.thread_id] });
                    }
                    queryClient.invalidateQueries({ queryKey: ["conversations"] });
                } catch (e) {
                    console.error('Failed to parse read receipt event data:', e);
                }
            });

            es.addEventListener("new_broadcast", () => {
                console.log('New broadcast event received');
                queryClient.invalidateQueries({ queryKey: ["broadcasts"] });
                
                // Play notification sound
                playNotificationSound();
                
                // Show browser notification if enabled
                if (Notification.permission === "granted") {
                    const notification = new Notification("New Broadcast", {
                        body: "A new broadcast has been posted",
                        icon: "/favicon.ico",
                        tag: "new-broadcast",
                        requireInteraction: false,
                    });
                    
                    // Handle notification click - focus window and navigate to broadcasts
                    notification.onclick = () => {
                        window.focus();
                        window.location.href = '/dashboard/broadcasts';
                        notification.close();
                    };
                }
            });

            es.addEventListener("new_comment", (event) => {
                console.log('New comment event received:', event.data);
                try {
                    const data = JSON.parse(event.data);
                    if (data.broadcast_id) {
                        queryClient.invalidateQueries({ queryKey: ["broadcast-comments", data.broadcast_id] });
                    }
                    queryClient.invalidateQueries({ queryKey: ["broadcasts"] });
                } catch (e) {
                    console.error('Failed to parse comment event data:', e);
                }
            });

            es.onerror = (error) => {
                console.error('SSE connection error:', error);
                // On error, close and reconnect after 3 seconds
                es.close();
                esRef.current = null;
                reconnectTimeoutRef.current = setTimeout(connect, 3000);
            };
        }

        connect();

        // Request notification permission
        if (Notification.permission === "default") {
            Notification.requestPermission().then(permission => {
                console.log('Notification permission:', permission);
            });
        }

        return () => {
            if (reconnectTimeoutRef.current) {
                clearTimeout(reconnectTimeoutRef.current);
            }
            esRef.current?.close();
            esRef.current = null;
        };
    }, [queryClient, playNotificationSound]);
}

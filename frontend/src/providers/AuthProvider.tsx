import { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { auth, User } from '@/lib/api';

interface AuthContextType {
    user: User | null;
    isLoading: boolean;
    isAuthenticated: boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
    const [user, setUser] = useState<User | null>(null);
    const [isLoading, setIsLoading] = useState(true);

    useEffect(() => {
        // Set a timeout to prevent infinite loading
        const timeout = setTimeout(() => {
            console.warn('Auth check timed out, assuming not authenticated');
            setIsLoading(false);
        }, 5000);

        auth
            .getMe()
            .then((userData) => {
                setUser(userData);
                clearTimeout(timeout);
            })
            .catch((error) => {
                // Not authenticated - this is okay
                console.log('Not authenticated:', error.message);
                setUser(null);
                clearTimeout(timeout);
            })
            .finally(() => {
                setIsLoading(false);
            });

        return () => clearTimeout(timeout);
    }, []);

    return (
        <AuthContext.Provider
            value={{
                user,
                isLoading,
                isAuthenticated: user !== null,
            }}
        >
            {children}
        </AuthContext.Provider>
    );
}

export function useAuth() {
    const context = useContext(AuthContext);
    if (context === undefined) {
        throw new Error('useAuth must be used within an AuthProvider');
    }
    return context;
}

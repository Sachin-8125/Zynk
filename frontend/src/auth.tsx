import { createContext, useContext, ReactNode, useState, useEffect } from "react";
import  { api, User } from "./api";


interface AuthContext {
    user: User | null;
    loading: boolean;
    signIn: (token: string, user: User) => void;
    signOut: () => void;
}

const context = createContext<AuthContext>(null!);
export const useAuth = () => useContext(context);

export function AuthProvider({children}: {children: ReactNode}) {
    const [user, setUser] = useState<User | null>(null);
    const [loading, setLoading] = useState(!!localStorage.getItem("token"));

    useEffect(() => {
        if (!localStorage.getItem("token")) return;
        api.me()
            .then(setUser)
            .catch(() => localStorage.removeItem("token"))
            .finally(() => setLoading(false));
    },[]);

    const signIn = (token: string, u: User) => {
        localStorage.setItem("token", token);
        setUser(u);
    };

    const signOut = () => {
        localStorage.removeItem("token");
        setUser(null);
    };

    return <context.Provider value={{user, loading, signIn, signOut}}>{children}</context.Provider>
}
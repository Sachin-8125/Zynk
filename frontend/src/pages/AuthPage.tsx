import { api } from "../api";
import { useAuth } from "../auth";
import { useState, type FormEvent } from "react";
import { useNavigate } from "react-router-dom";


export default function AuthPage() {
    const [mode, setMode] = useState<'signin' | 'register'>('signin');
    const [form, setForm] = useState({username: '', email:'', password: ''});
    const [error, setError] = useState('');
    const [busy, setBusy] = useState(false);
    const {signIn} = useAuth();
    const navigate = useNavigate();

    const set = (k: keyof typeof form) => (e: React.ChangeEvent<HTMLInputElement>) =>
    setForm({ ...form, [k]: e.target.value });

    async function submit(e: FormEvent) {
        e.preventDefault();
        setBusy(true);
        setError("");
        try {
        const res =
            mode === "register"
            ? await api.register(form)
            : await api.login({ email: form.email, password: form.password });
        signIn(res.token, res.user);
        navigate("/posts");
        } catch (err) {
        setError((err as Error).message);
        } finally {
        setBusy(false);
        }
    }

    return (
        <form className="authform" onSubmit={submit}>
        <h1>{mode === "signin" ? "Sign in" : "Create your account"}</h1>
        {mode === "register" && (
            <label>Username<input value={form.username} onChange={set("username")} required autoComplete="username" /></label>
        )}
        <label>Email<input type="email" value={form.email} onChange={set("email")} required autoComplete="email" /></label>
        <label>Password<input type="password" value={form.password} onChange={set("password")} required minLength={8}
            autoComplete={mode === "signin" ? "current-password" : "new-password"} /></label>
        {error && <p className="error">{error}</p>}
        <button className="primary" disabled={busy}>{mode === "signin" ? "Sign in" : "Create account"}</button>
        <button type="button" className="link" onClick={() => setMode(mode === "signin" ? "register" : "signin")}>
            {mode === "signin" ? "New here? Create an account" : "Have an account? Sign in"}
        </button>
        </form>
    );
}
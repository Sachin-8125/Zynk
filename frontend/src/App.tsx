import { Link, NavLink, Navigate, Route, Routes, useNavigate } from "react-router-dom";
import  type { ReactNode }  from "react";
import { useAuth } from "./auth";
import Home from "./pages/Home";
import PostPage from "./pages/PostPage";
import AuthPage from "./pages/AuthPage";
import Dashboard from "./pages/Dashboard";
import Editor from "./pages/Editor";

function RequireAuth({ children }: { children: ReactNode }) {
  const { user, loading } = useAuth();
  if (loading) return <p className="muted">Loading…</p>;
  return user ? <>{children}</> : <Navigate to="/signin" replace />;
}

export default function App() {
  const { user, signOut } = useAuth();
  const navigate = useNavigate();

  return (
    <>
      <header className="masthead">
        <Link to="/" className="brand">Longhand</Link>
        <nav>
          {user ? (
            <>
              <NavLink to="/write">Write</NavLink>
              <NavLink to="/posts">Your posts</NavLink>
              <button className="link" onClick={() => { signOut(); navigate("/"); }}>
                Sign out
              </button>
            </>
          ) : (
            <NavLink to="/signin">Sign in</NavLink>
          )}
        </nav>
      </header>
      <main>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/p/:slug" element={<PostPage />} />
          <Route path="/signin" element={<AuthPage />} />
          <Route path="/posts" element={<RequireAuth><Dashboard /></RequireAuth>} />
          <Route path="/write" element={<RequireAuth><Editor /></RequireAuth>} />
          <Route path="/write/:id" element={<RequireAuth><Editor /></RequireAuth>} />
          <Route path="*" element={<p className="muted">That page doesn’t exist.</p>} />
        </Routes>
      </main>
    </>
  );
}

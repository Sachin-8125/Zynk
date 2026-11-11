import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { api, formatDate } from "../api";
import type { PostSummary } from "../api";

export default function Dashboard() {
  const [posts, setPosts] = useState<PostSummary[] | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    api.myPosts().then(setPosts).catch((e) => setError(e.message));
  }, []);

  async function remove(p: PostSummary) {
    if (!confirm(`Delete “${p.title}”? This can’t be undone.`)) return;
    try {
      await api.deletePost(p.id);
      setPosts((cur) => cur!.filter((x) => x.id !== p.id));
    } catch (e) {
      setError((e as Error).message);
    }
  }

  if (!posts) return error ? <p className="error">{error}</p> : <p className="muted">Loading…</p>;

  return (
    <>
      <div className="row">
        <h1>Your posts</h1>
        <Link to="/write" className="btn primary">New post</Link>
      </div>
      {error && <p className="error">{error}</p>}
      {posts.length === 0 && <p className="muted">You haven’t written anything yet.</p>}
      <ul className="manage">
        {posts.map((p) => (
          <li key={p.id}>
            <div>
              <strong>{p.title}</strong>
              <span className={p.published ? "tag live" : "tag"}>{p.published ? "Published" : "Draft"}</span>
              <div className="meta">{formatDate(p.created_at)}</div>
            </div>
            <div className="actions">
              {p.published && <Link to={`/p/${p.slug}`}>View</Link>}
              <Link to={`/write/${p.id}`}>Edit</Link>
              <button className="link danger" onClick={() => remove(p)}>Delete</button>
            </div>
          </li>
        ))}
      </ul>
    </>
  );
}

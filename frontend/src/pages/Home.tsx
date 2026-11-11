import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { api, formatDate } from "../api";
import type { PostList } from "../api";

export default function Home() {
  const [page, setPage] = useState(1);
  const [data, setData] = useState<PostList | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    api.listPosts(page).then(setData).catch((e) => setError(e.message));
  }, [page]);

  if (error) return <p className="error">{error}</p>;
  if (!data) return <p className="muted">Loading…</p>;
  if (data.posts.length === 0)
    return (
      <p className="muted">
        Nothing published yet. <Link to="/write">Write the first post.</Link>
      </p>
    );

  const lastPage = Math.max(1, Math.ceil(data.total / data.per_page));

  return (
    <>
      <ol className="postlist">
        {data.posts.map((p) => (
          <li key={p.id}>
            <Link to={`/p/${p.slug}`}>
              <h2>{p.title}</h2>
              <p>{p.excerpt}</p>
              <span className="meta">{p.author}, {formatDate(p.created_at)}</span>
            </Link>
          </li>
        ))}
      </ol>
      {lastPage > 1 && (
        <div className="pager">
          <button disabled={page <= 1} onClick={() => setPage(page - 1)}>Newer</button>
          <span className="muted">Page {page} of {lastPage}</span>
          <button disabled={page >= lastPage} onClick={() => setPage(page + 1)}>Older</button>
        </div>
      )}
    </>
  );
}

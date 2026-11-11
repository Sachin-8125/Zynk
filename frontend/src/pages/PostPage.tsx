import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import ReactMarkdown from "react-markdown";
import { api, formatDate } from "../api";
import type { Post } from "../api";

export default function PostPage() {
  const { slug } = useParams();
  const [post, setPost] = useState<Post | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    api.getPost(slug!).then(setPost).catch((e) => setError(e.message));
  }, [slug]);

  if (error) return <p className="error">{error}</p>;
  if (!post) return <p className="muted">Loading…</p>;

  return (
    <article className="article">
      <h1>{post.title}</h1>
      <p className="meta">{post.author}, {formatDate(post.created_at)}</p>
      <div className="prose"><ReactMarkdown>{post.body}</ReactMarkdown></div>
    </article>
  );
}

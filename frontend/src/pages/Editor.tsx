import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import ReactMarkdown from "react-markdown";
import { api } from "../api";

export default function Editor() {
  const { id } = useParams();
  const navigate = useNavigate();
  const [title, setTitle] = useState("");
  const [excerpt, setExcerpt] = useState("");
  const [body, setBody] = useState("");
  const [preview, setPreview] = useState(false);
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!id) return;
    api.getOwn(id)
      .then((p) => { setTitle(p.title); setExcerpt(p.excerpt); setBody(p.body); })
      .catch((e) => setError(e.message));
  }, [id]);

  async function save(published: boolean) {
    setBusy(true);
    setError("");
    setStatus("");
    const input = { title, body, excerpt, published };
    try {
      if (id) {
        await api.updatePost(id, input);
        setStatus(published ? "Published." : "Draft saved.");
      } else {
        const created = await api.createPost(input);
        navigate(`/write/${created.id}`, { replace: true });
      }
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="editor">
      <input className="title-input" placeholder="Title" value={title} onChange={(e) => setTitle(e.target.value)} />
      <input className="excerpt-input" placeholder="Short summary (optional)" value={excerpt}
        onChange={(e) => setExcerpt(e.target.value)} />
      <div className="tabs">
        <button className={!preview ? "active" : ""} onClick={() => setPreview(false)}>Write</button>
        <button className={preview ? "active" : ""} onClick={() => setPreview(true)}>Preview</button>
        <span className="muted">Markdown supported</span>
      </div>
      {preview ? (
        <div className="prose preview"><ReactMarkdown>{body || "*Nothing to preview yet.*"}</ReactMarkdown></div>
      ) : (
        <textarea value={body} onChange={(e) => setBody(e.target.value)} placeholder="Start writing…" rows={20} />
      )}
      {error && <p className="error">{error}</p>}
      {status && <p className="ok">{status}</p>}
      <div className="row">
        <button disabled={busy} onClick={() => save(false)}>Save draft</button>
        <button className="primary" disabled={busy} onClick={() => save(true)}>Publish</button>
      </div>
    </div>
  );
}

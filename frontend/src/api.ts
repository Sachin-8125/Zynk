export interface User {
  id: string;
  username: string;
}

export interface PostSummary {
  id: string;
  author: string;
  title: string;
  slug: string;
  excerpt: string;
  published: boolean;
  created_at: string;
}

export interface Post extends PostSummary {
  author_id: string;
  body: string;
  updated_at: string;
}

export interface PostList {
  posts: PostSummary[];
  page: number;
  per_page: number;
  total: number;
}

export interface PostInput {
  title: string;
  body: string;
  excerpt?: string;
  published: boolean;
}

const BASE = import.meta.env.VITE_API_URL ?? "";

async function req<T>(path: string, opts: RequestInit = {}): Promise<T> {
  const token = localStorage.getItem("token");
  const res = await fetch(`${BASE}${path}`, {
    ...opts,
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...opts.headers,
    },
  });
  if (res.status === 204) return undefined as T;
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.error ?? "Something went wrong");
  return data as T;
}

export const api = {
  register: (b: { username: string; email: string; password: string }) =>
    req<{ token: string; user: User }>("/api/auth/register", { method: "POST", body: JSON.stringify(b) }),
  login: (b: { email: string; password: string }) =>
    req<{ token: string; user: User }>("/api/auth/login", { method: "POST", body: JSON.stringify(b) }),
  me: () => req<User>("/api/auth/me"),

  listPosts: (page = 1) => req<PostList>(`/api/posts?page=${page}`),
  getPost: (slug: string) => req<Post>(`/api/posts/${slug}`),

  myPosts: () => req<PostSummary[]>("/api/manage/posts"),
  getOwn: (id: string) => req<Post>(`/api/manage/posts/${id}`),
  createPost: (b: PostInput) => req<Post>("/api/manage/posts", { method: "POST", body: JSON.stringify(b) }),
  updatePost: (id: string, b: PostInput) =>
    req<Post>(`/api/manage/posts/${id}`, { method: "PUT", body: JSON.stringify(b) }),
  deletePost: (id: string) => req<void>(`/api/manage/posts/${id}`, { method: "DELETE" }),
};

export const formatDate = (iso: string) =>
  new Date(iso).toLocaleDateString(undefined, { year: "numeric", month: "long", day: "numeric" });

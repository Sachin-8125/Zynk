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
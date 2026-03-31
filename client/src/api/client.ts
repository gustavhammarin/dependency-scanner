const BASE_URL = import.meta.env.VITE_API_URL ?? "http://localhost:5000/api";

export async function apiFetch(
    path: string,
    options: RequestInit
){
    const res = await fetch(`${BASE_URL}${path}`, {
        headers: {
            ...options.headers
        },
        ...options
    });

    return res;
}

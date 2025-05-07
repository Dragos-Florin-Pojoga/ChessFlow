// authToken.ts

const TOKEN_KEY = "token";

/**
 * Stores a JWT token in either localStorage or sessionStorage.
 * @param token JWT token string
 * @param remember If true, uses localStorage; otherwise sessionStorage
 */
export function setToken(token: string, remember: boolean): void {
    if (remember) {
        localStorage.setItem(TOKEN_KEY, token);
        sessionStorage.removeItem(TOKEN_KEY);
    } else {
        sessionStorage.setItem(TOKEN_KEY, token);
        localStorage.removeItem(TOKEN_KEY);
    }
}

/**
 * Retrieves the JWT token from either storage.
 * @returns Token string or null if not found
 */
export function getToken(): string | null {
    return localStorage.getItem(TOKEN_KEY) || sessionStorage.getItem(TOKEN_KEY);
}

/**
 * Clears the JWT token from both localStorage and sessionStorage.
 */
export function clearToken(): void {
    localStorage.removeItem(TOKEN_KEY);
    sessionStorage.removeItem(TOKEN_KEY);
}

/**
 * Checks if a token exists.
 * @returns True if a token is found in either storage
 */
export function isLoggedIn(): boolean {
    return !!getToken();
}

/**
 * Decodes a JWT token and returns its payload as an object.
 * Requires the token to be a valid base64-encoded JWT.
 */
export function decodeToken<T = any>(): T | null {
    const token = getToken();
    if (!token) return null;

    try {
        const payload = token.split(".")[1];
        const base64 = payload.replace(/-/g, "+").replace(/_/g, "/");
        const json = atob(base64);
        const parsed = JSON.parse(json);

        // Normalize the role claim key
        const roleClaimKey = "http://schemas.microsoft.com/ws/2008/06/identity/claims/role";
        const roles = parsed[roleClaimKey];

        // Inject a simplified 'role' key for easier client access
        if (roles) {
            parsed.role = roles;
        }

        return parsed;
    } catch {
        return null;
    }
}
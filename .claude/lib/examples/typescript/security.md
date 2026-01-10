# TypeScript Security Patterns

## Input Validation

```typescript
import { z } from 'zod';

// Define schema at boundary
const CreateUserSchema = z.object({
  email: z.string().email(),
  name: z.string().min(2).max(100),
  password: z.string().min(8).max(128),
});

// Validate input
function createUser(input: unknown) {
  const parsed = CreateUserSchema.safeParse(input);
  if (!parsed.success) {
    throw new ValidationError(parsed.error.message);
  }
  // parsed.data is now typed and validated
  return userService.create(parsed.data);
}
```

## XSS Prevention

```typescript
// Svelte auto-escapes by default, but be careful with {@html}
// Bad
{@html userInput}

// Good - use text content
<p>{userInput}</p>

// If HTML needed, sanitize first
import DOMPurify from 'dompurify';
{@html DOMPurify.sanitize(userInput)}
```

## CSRF Protection

```typescript
// SvelteKit form action with CSRF
// In +page.server.ts
export const actions = {
  default: async ({ request, cookies }) => {
    // SvelteKit validates CSRF automatically for form actions
    const data = await request.formData();
    // Process safely...
  }
};

// For API calls, include token
const response = await fetch('/api/action', {
  method: 'POST',
  headers: {
    'X-CSRF-Token': csrfToken,
  },
  body: JSON.stringify(data),
});
```

## Authentication Token Handling

```typescript
// Never store tokens in localStorage (XSS vulnerable)
// Use httpOnly cookies set by server

// Good: Cookie set by server
// Set-Cookie: session=xxx; HttpOnly; Secure; SameSite=Strict

// Client just sends credentials
await fetch('/api/protected', {
  credentials: 'include',  // Sends cookies
});
```

## Secure API Calls

```typescript
// Always validate response structure
const response = await fetch('/api/user');
if (!response.ok) {
  throw new ApiError(response.status);
}

const data = await response.json();

// Validate data matches expected shape
const user = UserSchema.parse(data);
```

## URL Parameter Handling

```typescript
// Validate route parameters
import { z } from 'zod';

const uuidSchema = z.string().uuid();

export async function load({ params }) {
  const result = uuidSchema.safeParse(params.id);
  if (!result.success) {
    throw error(400, 'Invalid ID format');
  }

  // Safe to use params.id
  return { user: await getUser(result.data) };
}
```

## Sensitive Data in State

```typescript
// Don't store sensitive data in client state
// Bad
const [password, setPassword] = useState('');

// Sensitive data should be:
// 1. Sent directly to server
// 2. Never stored in state longer than needed
// 3. Cleared after use

const handleSubmit = async (e: FormEvent) => {
  const formData = new FormData(e.currentTarget);
  await submitToServer(formData);  // Password goes directly to server
  e.currentTarget.reset();  // Clear form
};
```

## Content Security Policy

```typescript
// In SvelteKit hooks.server.ts
export const handle: Handle = async ({ event, resolve }) => {
  const response = await resolve(event);

  response.headers.set(
    'Content-Security-Policy',
    "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';"
  );

  return response;
};
```

## Rate Limiting (Client-Side Debounce)

```typescript
import { debounce } from 'lodash-es';

// Prevent rapid API calls
const debouncedSearch = debounce(async (query: string) => {
  const results = await searchApi(query);
  setResults(results);
}, 300);

// In input handler
function handleSearch(e: Event) {
  const query = (e.target as HTMLInputElement).value;
  debouncedSearch(query);
}
```

## Environment Variables

```typescript
// Only VITE_* vars are exposed to client
// In .env
DATABASE_URL=xxx          # Server only
VITE_API_URL=https://...  # Exposed to client

// Access safely
const apiUrl = import.meta.env.VITE_API_URL;

// Never expose secrets to client
// BAD: VITE_SECRET_KEY=xxx
```

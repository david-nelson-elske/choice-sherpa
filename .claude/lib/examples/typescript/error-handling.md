# TypeScript Error Handling Patterns

## Result Pattern (Recommended for Domain)

```typescript
type Result<T, E = Error> =
  | { ok: true; value: T }
  | { ok: false; error: E };

// Usage
function validateEmail(email: string): Result<Email, ValidationError> {
  if (!email) {
    return { ok: false, error: new ValidationError('Email required') };
  }
  if (!email.includes('@')) {
    return { ok: false, error: new ValidationError('Invalid format') };
  }
  return { ok: true, value: new Email(email) };
}

// Consuming
const result = validateEmail(input);
if (!result.ok) {
  console.error(result.error.message);
  return;
}
const email = result.value;
```

## Custom Error Classes

```typescript
export class DomainError extends Error {
  constructor(message: string) {
    super(message);
    this.name = this.constructor.name;
  }
}

export class NotFoundError extends DomainError {
  constructor(public readonly resourceType: string, public readonly id: string) {
    super(`${resourceType} not found: ${id}`);
  }
}

export class ValidationError extends DomainError {
  constructor(public readonly field: string, public readonly reason: string) {
    super(`Invalid ${field}: ${reason}`);
  }
}

export class AuthorizationError extends DomainError {
  constructor(public readonly action: string, public readonly resource: string) {
    super(`Not authorized to ${action} ${resource}`);
  }
}
```

## Try-Catch Patterns

```typescript
// Specific error handling
try {
  await service.createUser(input);
} catch (error) {
  if (error instanceof ValidationError) {
    return res.status(400).json({ error: error.message });
  }
  if (error instanceof NotFoundError) {
    return res.status(404).json({ error: error.message });
  }
  // Re-throw unexpected errors
  throw error;
}

// With type guard
function isApiError(error: unknown): error is ApiError {
  return error instanceof Error && 'statusCode' in error;
}
```

## Async Error Handling

```typescript
// With try-catch
async function fetchUser(id: string): Promise<User> {
  try {
    const response = await fetch(`/api/users/${id}`);
    if (!response.ok) {
      throw new ApiError(`HTTP ${response.status}`, response.status);
    }
    return await response.json();
  } catch (error) {
    if (error instanceof ApiError) throw error;
    throw new NetworkError('Failed to fetch user');
  }
}

// With Result type
async function fetchUserSafe(id: string): Promise<Result<User, FetchError>> {
  try {
    const user = await fetchUser(id);
    return { ok: true, value: user };
  } catch (error) {
    return { ok: false, error: error as FetchError };
  }
}
```

## Validation Helpers

```typescript
// Collect all errors
interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
}

function validateUser(input: UserInput): ValidationResult {
  const errors: ValidationError[] = [];

  if (!input.email) {
    errors.push(new ValidationError('email', 'required'));
  } else if (!isValidEmail(input.email)) {
    errors.push(new ValidationError('email', 'invalid format'));
  }

  if (!input.name || input.name.length < 2) {
    errors.push(new ValidationError('name', 'must be at least 2 characters'));
  }

  return { valid: errors.length === 0, errors };
}
```

## Never Throw in Domain Logic

```typescript
// Bad: throws, caller doesn't know
function getUser(id: string): User {
  const user = repo.find(id);
  if (!user) throw new NotFoundError('User', id);  // Unexpected!
  return user;
}

// Good: return type indicates possible failure
function getUser(id: string): Result<User, NotFoundError> {
  const user = repo.find(id);
  if (!user) return { ok: false, error: new NotFoundError('User', id) };
  return { ok: true, value: user };
}

// Good: Optional for find operations
function findUser(id: string): User | undefined {
  return repo.find(id);
}
```

## Error Mapping at Boundaries

```typescript
// API adapter maps domain errors to HTTP
function handleError(error: unknown): Response {
  if (error instanceof ValidationError) {
    return json({ error: error.message }, { status: 400 });
  }
  if (error instanceof NotFoundError) {
    return json({ error: error.message }, { status: 404 });
  }
  if (error instanceof AuthorizationError) {
    return json({ error: error.message }, { status: 403 });
  }

  // Log unexpected errors
  console.error('Unexpected error:', error);
  return json({ error: 'Internal server error' }, { status: 500 });
}
```

## Optional Chaining and Nullish Coalescing

```typescript
// Safe property access
const name = user?.profile?.displayName ?? 'Anonymous';

// With default
const count = response.data?.items?.length ?? 0;

// In assignments
this.cache ??= new Map();  // Only assign if null/undefined
```

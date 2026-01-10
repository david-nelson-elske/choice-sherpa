# TypeScript Common Patterns

## Type Definitions

```typescript
// Interfaces for objects
interface User {
  id: string;
  email: string;
  name?: string;  // Optional
}

// Type aliases for unions and primitives
type Status = 'pending' | 'active' | 'archived';
type UserId = string;

// Readonly for immutability
interface Session {
  readonly id: string;
  readonly createdAt: Date;
  status: Status;  // Mutable
}
```

## Discriminated Unions

```typescript
type Result<T, E = Error> =
  | { ok: true; value: T }
  | { ok: false; error: E };

// TypeScript narrows based on discriminant
function handleResult(result: Result<User>) {
  if (result.ok) {
    console.log(result.value.name);  // TypeScript knows value exists
  } else {
    console.error(result.error.message);  // TypeScript knows error exists
  }
}
```

## Generic Types

```typescript
// Generic function
function first<T>(items: T[]): T | undefined {
  return items[0];
}

// Generic interface
interface Repository<T, ID = string> {
  find(id: ID): Promise<T | undefined>;
  save(entity: T): Promise<void>;
  delete(id: ID): Promise<void>;
}

// Constrained generic
function getId<T extends { id: string }>(entity: T): string {
  return entity.id;
}
```

## Utility Types

```typescript
// Partial - all properties optional
type UpdateUser = Partial<User>;

// Required - all properties required
type CompleteUser = Required<User>;

// Pick - subset of properties
type UserSummary = Pick<User, 'id' | 'name'>;

// Omit - exclude properties
type CreateUser = Omit<User, 'id' | 'createdAt'>;

// Record - dictionary type
type StatusCounts = Record<Status, number>;

// Readonly - immutable version
type ImmutableUser = Readonly<User>;
```

## Class Patterns

```typescript
class UserService {
  constructor(
    private readonly repository: UserRepository,
    private readonly eventBus: EventBus,
  ) {}

  async create(input: CreateUserInput): Promise<User> {
    const user = User.create(input);
    await this.repository.save(user);
    await this.eventBus.publish(new UserCreatedEvent(user));
    return user;
  }
}
```

## Factory Functions

```typescript
// Prefer factory functions for domain objects
function createUser(input: CreateUserInput): User {
  return {
    id: crypto.randomUUID(),
    email: input.email,
    name: input.name,
    createdAt: new Date(),
    status: 'pending',
  };
}

// With validation
function createSession(input: CreateSessionInput): Result<Session, ValidationError> {
  if (!input.title.trim()) {
    return { ok: false, error: new ValidationError('title', 'required') };
  }
  return {
    ok: true,
    value: {
      id: crypto.randomUUID(),
      title: input.title.trim(),
      createdAt: new Date(),
    },
  };
}
```

## Async Patterns

```typescript
// Parallel execution
const [users, sessions] = await Promise.all([
  userService.getAll(),
  sessionService.getAll(),
]);

// Sequential with reduce
const results = await items.reduce(
  async (accPromise, item) => {
    const acc = await accPromise;
    const result = await processItem(item);
    return [...acc, result];
  },
  Promise.resolve([] as Result[])
);

// With error handling
const results = await Promise.allSettled(promises);
const successes = results
  .filter((r): r is PromiseFulfilledResult<T> => r.status === 'fulfilled')
  .map(r => r.value);
```

## Svelte Store Patterns

```typescript
import { writable, derived, readable } from 'svelte/store';

// Writable store
const user = writable<User | null>(null);

// Derived store
const isLoggedIn = derived(user, $user => $user !== null);

// Custom store with methods
function createCounterStore() {
  const { subscribe, set, update } = writable(0);

  return {
    subscribe,
    increment: () => update(n => n + 1),
    decrement: () => update(n => n - 1),
    reset: () => set(0),
  };
}
```

## Type Guards

```typescript
// Custom type guard
function isUser(value: unknown): value is User {
  return (
    typeof value === 'object' &&
    value !== null &&
    'id' in value &&
    'email' in value
  );
}

// Usage
if (isUser(data)) {
  console.log(data.email);  // TypeScript knows it's User
}

// Assert function
function assertUser(value: unknown): asserts value is User {
  if (!isUser(value)) {
    throw new Error('Not a user');
  }
}
```

## Exhaustive Matching

```typescript
type Status = 'draft' | 'active' | 'archived';

function getStatusLabel(status: Status): string {
  switch (status) {
    case 'draft':
      return 'Draft';
    case 'active':
      return 'Active';
    case 'archived':
      return 'Archived';
    default:
      // Compile error if cases are missing
      const _exhaustive: never = status;
      throw new Error(`Unknown status: ${_exhaustive}`);
  }
}
```

## Module Organization

```typescript
// index.ts re-exports public API
export { UserService } from './user-service';
export { type User, type CreateUserInput } from './types';

// Internal implementation hidden
// ./internal/... not exported
```

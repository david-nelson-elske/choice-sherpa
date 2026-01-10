# TypeScript Testing Patterns (Vitest)

## Test Naming Convention

```typescript
describe('Subject', () => {
  it('should <expected> when <condition>', () => {
    // ...
  });
});

// Examples:
it('should return error when email is invalid')
it('should calculate sum when adding positive values')
it('should throw when session not found')
```

## AAA Pattern

```typescript
it('should validate email format', () => {
  // Arrange
  const user = new User('test@example.com');

  // Act
  const result = user.validate();

  // Assert
  expect(result.isValid).toBe(true);
});
```

## Common Assertions

```typescript
// Equality
expect(actual).toBe(expected);           // Strict equality (===)
expect(actual).toEqual(expected);        // Deep equality
expect(actual).not.toBe(unexpected);

// Truthiness
expect(value).toBeTruthy();
expect(value).toBeFalsy();
expect(value).toBeNull();
expect(value).toBeUndefined();
expect(value).toBeDefined();

// Numbers
expect(value).toBeGreaterThan(3);
expect(value).toBeLessThanOrEqual(10);
expect(value).toBeCloseTo(0.3, 5);       // Floating point

// Strings
expect(str).toContain('substring');
expect(str).toMatch(/pattern/);

// Arrays
expect(arr).toContain(item);
expect(arr).toHaveLength(3);

// Objects
expect(obj).toHaveProperty('key');
expect(obj).toMatchObject({ partial: 'match' });
```

## Testing Errors

```typescript
// Sync errors
expect(() => doSomething()).toThrow();
expect(() => doSomething()).toThrow('specific message');
expect(() => doSomething()).toThrow(SpecificError);

// Async errors
await expect(asyncFn()).rejects.toThrow();
await expect(asyncFn()).rejects.toThrow(NotFoundError);
```

## Testing Async Code

```typescript
it('should fetch user data', async () => {
  const service = new UserService(mockRepo);

  const result = await service.getUser('123');

  expect(result).toBeDefined();
  expect(result.id).toBe('123');
});

// With timeout
it('should complete within time limit', async () => {
  await expect(slowOperation()).resolves.toBeDefined();
}, 5000);
```

## Mocking with Vitest

```typescript
import { vi, Mock } from 'vitest';

// Mock function
const mockFn = vi.fn();
mockFn.mockReturnValue('result');
mockFn.mockResolvedValue('async result');

// Mock implementation
const mockFetch = vi.fn().mockImplementation((id: string) => {
  return Promise.resolve({ id, name: 'Test' });
});

// Verify calls
expect(mockFn).toHaveBeenCalled();
expect(mockFn).toHaveBeenCalledWith('arg1', 'arg2');
expect(mockFn).toHaveBeenCalledTimes(2);

// Mock module
vi.mock('./myModule', () => ({
  myFunction: vi.fn().mockReturnValue('mocked'),
}));
```

## Test Setup/Teardown

```typescript
describe('UserService', () => {
  let service: UserService;
  let mockRepo: Mock;

  beforeEach(() => {
    mockRepo = {
      find: vi.fn(),
      save: vi.fn(),
    };
    service = new UserService(mockRepo);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('should work', () => {
    // service is fresh for each test
  });
});
```

## Testing Components (Svelte)

```typescript
import { render, screen, fireEvent } from '@testing-library/svelte';
import MyComponent from './MyComponent.svelte';

it('should render title', () => {
  render(MyComponent, { props: { title: 'Hello' } });

  expect(screen.getByText('Hello')).toBeInTheDocument();
});

it('should handle click', async () => {
  const { component } = render(MyComponent);
  const button = screen.getByRole('button');

  await fireEvent.click(button);

  expect(screen.getByText('Clicked')).toBeInTheDocument();
});
```

## Testing API Responses

```typescript
it('should return user from API', async () => {
  const mockFetch = vi.fn().mockResolvedValue({
    ok: true,
    json: () => Promise.resolve({ id: '1', name: 'Test' }),
  });
  global.fetch = mockFetch;

  const result = await apiClient.getUser('1');

  expect(result.name).toBe('Test');
  expect(mockFetch).toHaveBeenCalledWith(
    expect.stringContaining('/users/1'),
    expect.any(Object)
  );
});
```

## Snapshot Testing

```typescript
it('should match snapshot', () => {
  const result = formatUserProfile(user);

  expect(result).toMatchSnapshot();
});

// Inline snapshot
it('should match inline snapshot', () => {
  expect(formatDate(date)).toMatchInlineSnapshot(`"2024-01-15"`);
});
```

## Test Organization

```typescript
describe('CycleService', () => {
  describe('create', () => {
    it('should create cycle with valid input', () => {});
    it('should throw when session not found', () => {});
  });

  describe('complete', () => {
    it('should complete in-progress cycle', () => {});
    it('should throw when already completed', () => {});
  });
});
```

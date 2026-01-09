# Feature: User Authentication

> Allow users to register and login with email/password, receiving JWT tokens for authenticated requests.

## Context

- Passwords must be hashed with bcrypt (cost factor 10)
- JWT tokens expire in 24 hours
- Use existing User model if available, otherwise create one
- Return proper HTTP status codes (201 for register, 200 for login, 401 for invalid credentials)

## Tasks

- [ ] Create password hashing utility with validation
- [ ] Create User entity/model with email validation
- [ ] Add register method to AuthService
- [ ] Add login method to AuthService
- [ ] Create POST /auth/register endpoint
- [ ] Create POST /auth/login endpoint

## Acceptance Criteria

- [ ] Passwords are never stored in plain text
- [ ] Duplicate email registration returns 409 Conflict
- [ ] Invalid login returns 401 Unauthorized (no info leak)
- [ ] Successful login returns JWT token
- [ ] JWT token contains user ID and expiration
- [ ] All endpoints have request validation

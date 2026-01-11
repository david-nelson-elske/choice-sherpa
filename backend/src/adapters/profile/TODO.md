# Profile Adapters - TODO

## Current Status

The profile adapters skeleton has been created but requires completion:

### ✅ Completed
- FsProfileStorage - Fully implemented with tests
- PgProfileRepository - Structure complete
- PgProfileReader - Structure complete
- Module integration

### ⚠️ Issues to Fix

#### 1. DecisionProfile Reconstruction
The `PgProfileRepository::from_db_row()` method cannot reconstruct a DecisionProfile from database fields because:
- Profile fields are private
- No `from_parts()` constructor exists
- `new()` creates an empty profile

**Solution**: Add a reconstruction method to DecisionProfile:
```rust
impl DecisionProfile {
    pub(crate) fn from_parts(
        id: DecisionProfileId,
        user_id: UserId,
        risk_profile: RiskProfile,
        // ... all other fields
    ) -> Self {
        // Direct construction
    }
}
```

#### 2. SQLx Query Macros
The typed query macros (`sqlx::query!`) require DATABASE_URL at compile time.

**Options**:
- Set DATABASE_URL in .env
- Use untyped `sqlx::query()` instead
- Run `cargo sqlx prepare` to cache queries

#### 3. Error Conversions
Several Result types need proper error mapping:
- `DecisionProfile::new()` returns `Result<_, String>`
- Need `.map_err(|e| DomainError::new(ErrorCode::ValidationFailed, e))`

#### 4. Markdown Generation
`PgProfileRepository::create()` and `update()` use placeholder markdown.

**Solution**: Implement proper markdown generator using profile data.

### 📝 Next Steps

1. Add `from_parts()` to DecisionProfile
2. Fix error conversions throughout
3. Implement ProfileAnalyzer
4. Add integration tests
5. Complete application layer
6. Add HTTP handlers

### 🔍 Files Modified

- `src/adapters/profile/filesystem.rs` - ✅ Complete
- `src/adapters/profile/postgres_repository.rs` - ⚠️ Needs fixes
- `src/adapters/profile/postgres_reader.rs` - ⚠️ Needs fixes
- `src/adapters/profile/mod.rs` - ✅ Complete
- `src/adapters/mod.rs` - ✅ Updated
- `Cargo.toml` - ✅ Added tempfile dependency

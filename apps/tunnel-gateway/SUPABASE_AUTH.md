# Supabase Authentication Setup

## Overview

The tunnel-gateway now uses **Supabase Auth** instead of custom JWT implementation. This provides:

- Managed authentication with secure token handling
- Email verification and password reset flows
- Built-in rate limiting and security features
- User management via Supabase dashboard
- Native PostgreSQL integration with pgvector

## Configuration

### Environment Variables

Add these to your `.env` file:

```env
# Database
DATABASE_URL=postgres://postgres:[PASSWORD]@db.[PROJECT_REF].supabase.co:5432/postgres

# Supabase Auth
SUPABASE_URL=https://[PROJECT_REF].supabase.co
SUPABASE_ANON_KEY=[YOUR_ANON_KEY]
SUPABASE_SERVICE_ROLE_KEY=[YOUR_SERVICE_ROLE_KEY]
```

### Getting Supabase Credentials

1. Go to [supabase.com](https://supabase.com) and create a project
2. Navigate to **Project Settings** → **API**
3. Copy:
   - **Project URL** → `SUPABASE_URL`
   - **anon/public key** → `SUPABASE_ANON_KEY`
   - **service_role key** → `SUPABASE_SERVICE_ROLE_KEY`
4. Navigate to **Project Settings** → **Database**
5. Copy **Connection string** → `DATABASE_URL`

## Architecture Changes

### Before (Custom JWT)
- Local SQLite database for users
- Custom bcrypt password hashing
- Manual JWT token generation and validation
- No email verification

### After (Supabase Auth)
- Supabase manages user authentication
- Tokens validated via Supabase API
- Built-in email verification
- User data in `auth.users` table
- Custom metadata (username) in `user_metadata` JSON field

## Migration Notes

### Removed Files
- Custom JWT password hashing logic (still in `jwt.rs` but unused)
- `user_db.rs` - Users now in Supabase `auth.users`

### New Files
- `auth/supabase_client.rs` - Supabase API wrapper

### Updated Files
- `auth/service.rs` - Uses SupabaseClient instead of JwtManager
- `auth/middleware.rs` - Validates Supabase tokens
- `main.rs` - Initializes SupabaseClient

## API Usage

### Registration
```rust
// Client sends:
RegisterRequest {
    username: "johndoe",
    email: "john@example.com",
    password: "SecurePass123!"
}

// Supabase creates user with:
// - email: "john@example.com"
// - user_metadata: { "username": "johndoe" }
```

### Login
```rust
// Client sends:
LoginRequest {
    username: "john@example.com",  // Treated as email
    password: "SecurePass123!"
}

// Response includes:
LoginResponse {
    access_token: "eyJhbG...",  // Supabase JWT
    refresh_token: "v1.MR...",
    user_id: "uuid",
    username: "johndoe"
}
```

### Token Verification
```rust
// gRPC metadata:
authorization: "Bearer eyJhbG..."

// Middleware validates with Supabase and extracts:
AuthClaims {
    sub: "user-uuid",
    email: "john@example.com",
    role: "authenticated"
}
```

## Database Schema

Users are stored in Supabase's `auth.users` table:
```sql
-- Auto-managed by Supabase
CREATE TABLE auth.users (
    id UUID PRIMARY KEY,
    email TEXT UNIQUE,
    encrypted_password TEXT,
    email_confirmed_at TIMESTAMPTZ,
    raw_user_meta_data JSONB,  -- { "username": "johndoe" }
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);
```

Memories table remains in `public` schema:
```sql
CREATE TABLE public.memories (
    id UUID PRIMARY KEY,
    content TEXT,
    embedding vector(384),
    metadata JSONB,
    tags TEXT[],
    created_at BIGINT,
    updated_at BIGINT
);
```

## Security Considerations

1. **Service Role Key** - Keep private! Can bypass RLS policies
2. **Anon Key** - Safe for client-side use, respects RLS
3. **Token Expiration** - Supabase tokens expire after 1 hour
4. **Refresh Tokens** - Valid for 30 days, use to get new access tokens

## Testing

```bash
# Start the gateway
just dev-gateway

# Register a user
grpcurl -plaintext -d '{
  "username": "testuser",
  "email": "test@example.com",
  "password": "TestPass123!"
}' localhost:50051 auth.AuthService/Register

# Login
grpcurl -plaintext -d '{
  "username": "test@example.com",
  "password": "TestPass123!"
}' localhost:50051 auth.AuthService/Login
```

## Troubleshooting

### "SUPABASE_URL not set"
- Ensure `.env` file exists in project root
- Run `dotenv().ok()` loads environment variables

### "Invalid or expired token"
- Check token hasn't expired (1 hour lifetime)
- Use refresh token to get new access token
- Verify token is sent as `Bearer <token>`

### "User not found"
- Confirm email/password are correct
- Check Supabase dashboard → Authentication → Users

## Future Enhancements

- [ ] Implement email verification flow
- [ ] Add password reset functionality
- [ ] OAuth providers (Google, GitHub)
- [ ] Row-level security for memories table
- [ ] User profiles table linked to auth.users

# KYC/AML Integration Implementation

This document describes the KYC/AML compliance integration added to StelloVault.

## Overview

The KYC (Know Your Customer) / AML (Anti-Money Laundering) integration provides compliance hooks for real-world adoption. It includes:

1. Database schema changes to track KYC status
2. A placeholder KYC service that can be swapped for real providers (Sumsub, Persona, etc.)
3. Compliance middleware to enforce KYC requirements
4. API endpoints for KYC management

## Database Changes

### Migration: `006_add_kyc_fields.sql`

Adds the following fields to the `users` table:

- `kyc_status`: ENUM ('unverified', 'pending', 'verified', 'rejected', 'expired')
- `kyc_expiry`: TIMESTAMPTZ - When the KYC verification expires
- `kyc_provider`: TEXT - Name of the KYC provider (e.g., 'sumsub', 'persona')
- `kyc_verified_at`: TIMESTAMPTZ - When verification was completed
- `kyc_reference_id`: TEXT - Provider's reference ID for the verification

### Automatic Expiry

The migration includes:
- A trigger to automatically expire KYC on update
- A function `expire_kyc_statuses()` that can be called periodically via cron

## KYC Service

### Location: `backend/src/services/kyc.rs`

The `KycService` provides:

#### Core Methods

- `check_kyc_compliance(user_id, transaction_amount)` - Checks if user meets KYC requirements
  - Threshold: $10,000 (1,000,000 cents)
  - Returns `true` if compliant, `false` otherwise

- `get_kyc_status(user_id)` - Gets current KYC status for a user

- `initiate_verification(user_id)` - Starts KYC verification process

#### Mock Methods (for testing)

- `mock_approve_verification(user_id, provider, reference_id)` - Approves KYC (sets 1-year expiry)
- `mock_reject_verification(user_id, reason)` - Rejects KYC
- `expire_verification(user_id)` - Manually expires a user's KYC
- `expire_all_expired()` - Bulk expires all expired KYC verifications

### Integration Points

To integrate with real KYC providers (Sumsub, Persona, etc.):

1. Replace `initiate_verification()` with provider SDK calls
2. Add webhook handlers for provider callbacks
3. Update `mock_approve_verification()` to process real verification results
4. Add provider-specific configuration

## Compliance Middleware

### Location: `backend/src/middleware/compliance.rs`

The `kyc_compliance_middleware` function:

1. Extracts transaction amount from request body
2. Checks KYC compliance using `KycService`
3. Returns `403 Forbidden` with clear error message if KYC required
4. Allows request to proceed if compliant

### Error Response Format

```json
{
  "success": false,
  "error": "KYC verification required for transactions over $10,000. Please complete identity verification.",
  "code": "KYC_REQUIRED",
  "threshold": 1000000
}
```

## API Endpoints

### Location: `backend/src/handlers/kyc.rs`

#### User Endpoints

- `GET /api/kyc/status` - Get authenticated user's KYC status
- `POST /api/kyc/initiate` - Initiate KYC verification for authenticated user

#### Admin Endpoints

- `GET /api/kyc/users/:user_id/status` - Get KYC status for specific user
- `POST /api/kyc/users/:user_id/approve` - Mock approve KYC (testing only)
- `POST /api/kyc/users/:user_id/reject` - Mock reject KYC (testing only)

## Usage in Routes

To protect an endpoint with KYC compliance:

```rust
use axum::middleware;
use crate::middleware::kyc_compliance_middleware;

Router::new()
    .route("/api/escrows", post(create_escrow))
    .layer(middleware::from_fn_with_state(
        kyc_service.clone(),
        kyc_compliance_middleware
    ))
```

## Model Updates

### User Model

The `User` struct now includes:

```rust
pub struct User {
    // ... existing fields ...
    pub kyc_status: KycStatus,
    pub kyc_expiry: Option<DateTime<Utc>>,
    pub kyc_provider: Option<String>,
    pub kyc_verified_at: Option<DateTime<Utc>>,
    pub kyc_reference_id: Option<String>,
}
```

### UserResponse

The API response includes KYC status:

```rust
pub struct UserResponse {
    // ... existing fields ...
    pub kyc_status: KycStatus,
    pub kyc_expiry: Option<DateTime<Utc>>,
}
```

## Testing

### Mock Approval Flow

```bash
# 1. Check initial status
curl -X GET http://localhost:8080/api/kyc/status \
  -H "Authorization: Bearer <token>"

# 2. Initiate verification
curl -X POST http://localhost:8080/api/kyc/initiate \
  -H "Authorization: Bearer <token>"

# 3. Mock approve (admin only)
curl -X POST http://localhost:8080/api/kyc/users/<user_id>/approve \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "provider": "mock",
    "reference_id": "test-123"
  }'

# 4. Try creating high-value escrow
curl -X POST http://localhost:8080/api/escrows \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 1500000,
    ...
  }'
```

## Acceptance Criteria

✅ Users without KYC cannot create escrows > $10,000 (mock check)
✅ Backend returns clear 403 Forbidden with "KYC required" message
✅ KYC status and expiry added to User model
✅ ComplianceMiddleware blocks POST /api/escrows for unverified users
✅ Placeholder KycService ready for Sumsub/Persona integration

## Next Steps

1. **Frontend Integration**: Add "Verification Required" banner for unverified users
2. **Provider Integration**: Replace mock methods with real KYC provider SDK
3. **Webhook Handlers**: Add endpoints to receive provider callbacks
4. **Cron Job**: Set up periodic task to expire old KYC verifications
5. **Admin Dashboard**: Add UI for managing KYC verifications
6. **Audit Logging**: Track all KYC status changes for compliance

## Configuration

Add to `.env`:

```env
# KYC Provider Configuration (when integrating real provider)
KYC_PROVIDER=sumsub  # or 'persona'
KYC_API_KEY=your_api_key
KYC_API_SECRET=your_api_secret
KYC_WEBHOOK_SECRET=your_webhook_secret
KYC_THRESHOLD_CENTS=1000000  # $10,000
```

## Security Considerations

1. **PII Protection**: KYC data contains sensitive personal information
2. **Encryption**: Consider encrypting `kyc_reference_id` at rest
3. **Access Control**: Limit KYC status access to user and admins only
4. **Audit Trail**: Log all KYC status changes
5. **Data Retention**: Implement policies for KYC data retention and deletion
6. **Provider Security**: Ensure KYC provider meets compliance standards (SOC 2, ISO 27001)

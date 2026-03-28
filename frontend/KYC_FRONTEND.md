# KYC Frontend Implementation

This document describes the frontend components for KYC/AML compliance integration.

## Components

### 1. KycVerificationBanner

A reusable banner component that displays KYC verification status and prompts users to complete verification.

**Location**: `src/components/kyc/KycVerificationBanner.tsx`

**Features**:
- Displays different states: unverified, pending, verified, rejected, expired
- Color-coded by status (yellow for unverified, blue for pending, red for rejected, etc.)
- Dismissible
- Action button for initiating verification
- Responsive design with dark mode support

**Props**:
```typescript
interface KycVerificationBannerProps {
  kycStatus: 'unverified' | 'pending' | 'verified' | 'rejected' | 'expired';
  kycExpiry?: string | null;
  onInitiateVerification?: () => void;
  onDismiss?: () => void;
}
```

**Usage**:
```tsx
import { KycVerificationBanner } from '@/components/kyc';

<KycVerificationBanner
  kycStatus="unverified"
  onInitiateVerification={() => console.log('Start verification')}
/>
```

### 2. KycBannerContainer

A smart container component that fetches KYC status and displays the banner automatically.

**Location**: `src/components/kyc/KycBannerContainer.tsx`

**Features**:
- Automatically fetches KYC status on mount
- Handles loading states
- Only displays when verification is required
- Integrates with `useKycStatus` hook

**Usage**:
```tsx
import { KycBannerContainer } from '@/components/kyc';

// In your dashboard or layout
export default function Dashboard() {
  return (
    <div>
      <KycBannerContainer />
      {/* Rest of your dashboard */}
    </div>
  );
}
```

## Hooks

### useKycStatus

A custom hook for managing KYC status and verification flow.

**Location**: `src/hooks/useKycStatus.ts`

**Features**:
- Fetches current KYC status from API
- Provides method to initiate verification
- Handles loading and error states
- Auto-fetches on mount
- Provides refetch method for manual updates

**Return Value**:
```typescript
interface UseKycStatusReturn {
  kycStatus: KycStatus | null;
  kycExpiry: string | null;
  isValid: boolean;
  requiresVerification: boolean;
  isLoading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
  initiateVerification: () => Promise<void>;
}
```

**Usage**:
```tsx
import { useKycStatus } from '@/hooks/useKycStatus';

function MyComponent() {
  const {
    kycStatus,
    isValid,
    requiresVerification,
    initiateVerification,
  } = useKycStatus();

  if (requiresVerification) {
    return (
      <button onClick={initiateVerification}>
        Complete Verification
      </button>
    );
  }

  return <div>Verified!</div>;
}
```

## Integration Guide

### Step 1: Add to Dashboard Layout

Add the KYC banner to your main dashboard or layout:

```tsx
// app/dashboard/layout.tsx
import { KycBannerContainer } from '@/components/kyc';

export default function DashboardLayout({ children }) {
  return (
    <div className="dashboard">
      <KycBannerContainer />
      {children}
    </div>
  );
}
```

### Step 2: Protect High-Value Actions

Use the hook to check KYC status before allowing high-value transactions:

```tsx
// components/CreateEscrowForm.tsx
import { useKycStatus } from '@/hooks/useKycStatus';

export function CreateEscrowForm() {
  const { isValid, requiresVerification, initiateVerification } = useKycStatus();
  const [amount, setAmount] = useState(0);

  const handleSubmit = async () => {
    // Check if amount requires KYC
    if (amount > 1000000 && !isValid) { // $10,000 in cents
      alert('KYC verification required for transactions over $10,000');
      await initiateVerification();
      return;
    }

    // Proceed with escrow creation
    // ...
  };

  return (
    <form onSubmit={handleSubmit}>
      {/* Form fields */}
    </form>
  );
}
```

### Step 3: Handle API Errors

The backend will return 403 Forbidden for non-compliant transactions:

```tsx
try {
  const response = await fetch('/api/escrows', {
    method: 'POST',
    body: JSON.stringify(escrowData),
  });

  if (response.status === 403) {
    const error = await response.json();
    if (error.code === 'KYC_REQUIRED') {
      // Show KYC banner or redirect to verification
      setShowKycBanner(true);
      return;
    }
  }

  // Handle success
} catch (error) {
  // Handle error
}
```

## Styling

The components use Tailwind CSS with dark mode support. Key classes:

- **Unverified**: Yellow theme (`bg-yellow-50`, `text-yellow-600`)
- **Pending**: Blue theme (`bg-blue-50`, `text-blue-600`)
- **Rejected**: Red theme (`bg-red-50`, `text-red-600`)
- **Expired**: Orange theme (`bg-orange-50`, `text-orange-600`)

### Customization

To customize the banner appearance, modify the `getBannerConfig()` function in `KycVerificationBanner.tsx`:

```tsx
const getBannerConfig = () => {
  switch (kycStatus) {
    case 'unverified':
      return {
        // Customize colors, icons, messages
        bgColor: 'bg-custom-yellow',
        // ...
      };
  }
};
```

## API Integration

### Endpoints Used

1. **GET /api/kyc/status** - Fetch current KYC status
   ```typescript
   Response: {
     success: boolean;
     data: {
       user_id: string;
       kyc_status: KycStatus;
       kyc_expiry: string | null;
       is_valid: boolean;
       requires_verification: boolean;
     }
   }
   ```

2. **POST /api/kyc/initiate** - Start verification process
   ```typescript
   Response: {
     success: boolean;
     data: {
       user_id: string;
       kyc_status: KycStatus;
       message: string;
     }
   }
   ```

### Authentication

All API calls require an `Authorization` header with a Bearer token:

```typescript
headers: {
  'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
}
```

## Testing

### Manual Testing

1. **Unverified State**:
   - Create a new user
   - Banner should show "Verification Required"

2. **Pending State**:
   - Initiate verification
   - Banner should show "Verification Pending"

3. **Verified State**:
   - Admin approves KYC
   - Banner should disappear

4. **High-Value Transaction**:
   - Try creating escrow > $10,000 without KYC
   - Should receive 403 error

### Unit Tests

```tsx
// __tests__/KycVerificationBanner.test.tsx
import { render, screen } from '@testing-library/react';
import { KycVerificationBanner } from '@/components/kyc';

describe('KycVerificationBanner', () => {
  it('shows unverified message', () => {
    render(<KycVerificationBanner kycStatus="unverified" />);
    expect(screen.getByText('Verification Required')).toBeInTheDocument();
  });

  it('hides when verified', () => {
    const { container } = render(
      <KycVerificationBanner kycStatus="verified" />
    );
    expect(container.firstChild).toBeNull();
  });
});
```

## Accessibility

The components follow accessibility best practices:

- Semantic HTML with `role="alert"` for banners
- Proper ARIA labels for buttons
- Keyboard navigation support
- Screen reader friendly text
- Color contrast meets WCAG AA standards

## Next Steps

1. **Provider Integration**: Replace mock verification with real provider SDK
2. **Webhook Handling**: Add real-time status updates via WebSocket
3. **Document Upload**: Add UI for uploading verification documents
4. **Status History**: Show verification attempt history
5. **Email Notifications**: Notify users of status changes
6. **Admin Dashboard**: Add admin UI for managing verifications

## Dependencies

Required packages (already in Next.js):
- `lucide-react` - Icons
- `tailwindcss` - Styling

No additional dependencies needed!

## Browser Support

- Chrome/Edge: Latest 2 versions
- Firefox: Latest 2 versions
- Safari: Latest 2 versions
- Mobile browsers: iOS Safari 12+, Chrome Android

## Performance

- Components are client-side only (`'use client'`)
- KYC status cached in state
- Minimal re-renders
- Lazy loading ready
- No external API calls on every render

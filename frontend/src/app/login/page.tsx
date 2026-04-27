import { Suspense } from 'react';
import { Loader2 } from 'lucide-react';
import { LoginContent } from './LoginContent';

export default function LoginPage() {
  return (
    <Suspense
      fallback={
        <div className="min-h-screen flex items-center justify-center">
          <Loader2 className="w-6 h-6 animate-spin text-gray-400" />
        </div>
      }
    >
      <LoginContent />
    </Suspense>
  );
}

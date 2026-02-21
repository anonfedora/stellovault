import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { Keypair } from '@stellar/stellar-sdk';
import { setAuthCookies, signToken } from '@/lib/auth';

export async function POST(request: Request) {
    try {
        const { publicKey, signedMessage } = await request.json();
        const cookieStore = await cookies();
        const storedNonceData = cookieStore.get('auth-nonce')?.value;

        if (!storedNonceData) {
            return NextResponse.json({ error: 'No active challenge found' }, { status: 400 });
        }

        const { nonce, publicKey: boundedPublicKey } = JSON.parse(storedNonceData);

        if (publicKey !== boundedPublicKey) {
            return NextResponse.json({ error: 'Public key mismatch' }, { status: 400 });
        }

        if (!publicKey || !signedMessage) {
            return NextResponse.json({ error: 'Missing credentials' }, { status: 400 });
        }

        // Verify signature
        // The signed message from Freighter is the signature of the nonce
        // We need to verify that signature matches the nonce and public key

        // In Freighter, signMessage returns a signature (Buffer or base64 usually)
        // We need to verify it.
        // Keypair.fromPublicKey(publicKey).verify(data, signature)

        const keypair = Keypair.fromPublicKey(publicKey);
        const isValid = keypair.verify(Buffer.from(nonce), Buffer.from(signedMessage, 'base64'));

        if (!isValid) {
            return NextResponse.json({ error: 'Invalid signature' }, { status: 401 });
        }

        // Generate Tokens
        const accessToken = await signToken({ sub: publicKey }, '1h');
        const refreshToken = await signToken({ sub: publicKey, type: 'refresh' }, '7d');

        // Set Cookies
        await setAuthCookies(accessToken, refreshToken);

        // Clear nonce
        cookieStore.delete('auth-nonce');

        return NextResponse.json({ success: true });

    } catch (error) {
        console.error('Verify error:', error);
        return NextResponse.json({ error: 'Internal Server Error' }, { status: 500 });
    }
}

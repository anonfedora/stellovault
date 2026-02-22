export const runtime = 'nodejs';

import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { Keypair } from '@stellar/stellar-sdk';
import { setAuthCookies, signToken } from '@/lib/auth';

const NO_STORE = { 'Cache-Control': 'no-store' };

export async function POST(request: Request) {
    try {
        const { publicKey, signedMessage, signerPublicKey } = await request.json();

        const cookieStore = await cookies();
        const storedNonceData = cookieStore.get('auth-nonce')?.value;

        if (!storedNonceData) {
            return NextResponse.json({ error: 'No active challenge found' }, { status: 400, headers: NO_STORE });
        }

        const { nonce, publicKey: boundedPublicKey } = JSON.parse(storedNonceData);

        // Ensure request publicKey matches the nonce-bound key (and Freighter's reported signer)
        if (publicKey !== boundedPublicKey || (signerPublicKey && publicKey !== signerPublicKey)) {
            return NextResponse.json({ error: 'Public key mismatch' }, { status: 400, headers: NO_STORE });
        }

        if (!publicKey || !signedMessage) {
            return NextResponse.json({ error: 'Missing credentials' }, { status: 400, headers: NO_STORE });
        }

        // Verify Ed25519 signature (Node.js runtime — Buffer is safe here)
        const keypair = Keypair.fromPublicKey(publicKey);
        const isValid = keypair.verify(
            Buffer.from(nonce),
            Buffer.from(signedMessage, 'base64')
        );

        if (!isValid) {
            return NextResponse.json({ error: 'Invalid signature' }, { status: 401, headers: NO_STORE });
        }

        // Issue tokens
        const accessToken = await signToken({ sub: publicKey, type: 'access' }, '1h');
        const refreshToken = await signToken({ sub: publicKey, type: 'refresh' }, '7d');

        await setAuthCookies(accessToken, refreshToken);
        cookieStore.delete('auth-nonce');

        return NextResponse.json({ success: true }, { headers: NO_STORE });

    } catch (error) {
        console.error('[auth/verify] Verify error:', error);
        return NextResponse.json({ error: 'Internal Server Error' }, { status: 500, headers: NO_STORE });
    }
}

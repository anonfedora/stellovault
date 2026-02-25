import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import crypto from 'crypto';
import { StrKey } from '@stellar/stellar-sdk';

export async function POST(request: Request) {
    try {
        const { publicKey } = await request.json();

        if (!publicKey || !StrKey.isValidEd25519PublicKey(publicKey)) {
            return NextResponse.json({ error: 'Invalid public key' }, { status: 400 });
        }

        // Generate a random nonce
        const nonce = crypto.randomBytes(32).toString('hex');

        // Store nonce and publicKey in a temporary httpOnly cookie to bind them
        const cookieStore = await cookies();
        const cookieValue = JSON.stringify({ nonce, publicKey });

        cookieStore.set('auth-nonce', cookieValue, {
            httpOnly: true,
            secure: process.env.NODE_ENV === 'production',
            sameSite: 'strict',
            maxAge: 300, // 5 minutes
            path: '/',
        });

        return NextResponse.json({ nonce });
    } catch (error) {
        console.error('Challenge error:', error);
        return NextResponse.json({ error: 'Internal Server Error' }, { status: 500 });
    }
}

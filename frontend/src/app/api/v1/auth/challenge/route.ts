import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import crypto from 'crypto';

export async function POST(request: Request) {
    try {
        const { publicKey } = await request.json();

        if (!publicKey) {
            return NextResponse.json({ error: 'Public key required' }, { status: 400 });
        }

        // Generate a random nonce
        const nonce = crypto.randomBytes(32).toString('hex');

        // Store nonce in a temporary httpOnly cookie
        const cookieStore = await cookies();
        cookieStore.set('auth-nonce', nonce, {
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

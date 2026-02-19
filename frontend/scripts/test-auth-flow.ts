import { Keypair } from '@stellar/stellar-sdk';
// We need to use node-fetch or global fetch if node 18+
// usage: npx ts-node scripts/test-auth-flow.ts

async function testAuthFlow() {
    const BASE_URL = 'http://localhost:3000';

    console.log('1. Generating Keypair...');
    const keypair = Keypair.random();
    const publicKey = keypair.publicKey();
    console.log('   Public Key:', publicKey);

    try {
        // 1. Challenge
        console.log('\n2. Requesting Challenge...');
        const challengeRes = await fetch(`${BASE_URL}/api/v1/auth/challenge`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ publicKey }),
        });

        if (!challengeRes.ok) {
            throw new Error(`Challenge failed: ${challengeRes.status} ${await challengeRes.text()}`);
        }

        // Get cookie
        const cookieHeader = challengeRes.headers.get('set-cookie');
        console.log('   Cookies received:', cookieHeader ? 'Yes' : 'No');

        const { nonce } = await challengeRes.json() as { nonce: string };
        console.log('   Nonce:', nonce);

        // 2. Sign
        console.log('\n3. Signing Nonce...');
        // Freighter signs the message (string)
        // We sign the nonce (string)
        const signature = keypair.sign(Buffer.from(nonce));
        const signedMessage = signature.toString('base64');
        console.log('   Signature:', signedMessage.slice(0, 20) + '...');

        // 3. Verify
        console.log('\n4. Verifying...');
        const verifyRes = await fetch(`${BASE_URL}/api/v1/auth/verify`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Cookie': cookieHeader || ''
            },
            body: JSON.stringify({ publicKey, signedMessage }),
        });

        if (!verifyRes.ok) {
            throw new Error(`Verify failed: ${verifyRes.status} ${await verifyRes.text()}`);
        }

        const verifyData = await verifyRes.json();
        console.log('   Verification Result:', verifyData);

        const authCookies = verifyRes.headers.get('set-cookie');
        console.log('   Auth Cookies received:', authCookies ? 'Yes' : 'No');

        if (verifyData.success) {
            console.log('\n✅ Auth Flow Test PASSED');
        } else {
            console.error('\n❌ Auth Flow Test FAILED');
        }

    } catch (error) {
        console.error('\n❌ Test Error:', error);
    }
}

testAuthFlow();

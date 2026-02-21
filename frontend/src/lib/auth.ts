import { SignJWT, jwtVerify } from 'jose';
import { cookies } from 'next/headers';

const SECRET_KEY = process.env.JWT_SECRET_KEY;
if (!SECRET_KEY) {
  throw new Error('JWT_SECRET_KEY is not defined in environment variables');
}
const key = new TextEncoder().encode(SECRET_KEY);

export interface TokenPayload {
  sub: string;
  type?: 'access' | 'refresh';
  [key: string]: any;
}

export async function signToken(payload: TokenPayload, expiresIn: string = '1h') {
  return await new SignJWT(payload)
    .setProtectedHeader({ alg: 'HS256' })
    .setIssuedAt()
    .setExpirationTime(expiresIn)
    .sign(key);
}

export async function verifyToken(token: string) {
  try {
    const { payload } = await jwtVerify(token, key, {
      algorithms: ['HS256'],
    });
    return payload;
  } catch (error) {
    return null;
  }
}

export async function setAuthCookies(accessToken: string, refreshToken: string) {
  const cookieStore = await cookies();

  cookieStore.set('accessToken', accessToken, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'strict',
    maxAge: 3600, // 1 hour
    path: '/',
  });

  cookieStore.set('refreshToken', refreshToken, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'strict',
    maxAge: 60 * 60 * 24 * 7, // 7 days
    path: '/',
  });
}

export async function clearAuthCookies() {
  const cookieStore = await cookies();
  cookieStore.delete('accessToken');
  cookieStore.delete('refreshToken');
}

export async function refreshAccessToken(refreshToken: string): Promise<{ accessToken: string } | null> {
  const payload = await verifyToken(refreshToken);
  if (!payload || payload.type !== 'refresh') {
    return null;
  }

  const newAccessToken = await signToken({ sub: payload.sub as string, type: 'access' }, '1h');
  return { accessToken: newAccessToken };
}

export async function getSession() {
  const cookieStore = await cookies();
  const token = cookieStore.get('accessToken')?.value;
  if (!token) return null;
  return verifyToken(token);
}

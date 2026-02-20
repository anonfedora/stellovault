import {
  Keypair,
  StrKey,
} from "@stellar/stellar-sdk";

export function buildLoginMessage(nonce: string) {
  return `stellovault:login:${nonce}`;
}

export function verifySignature(params: {
  address: string;
  message: string;
  signature: string;
}) {
  const { address, message, signature } = params;

  if (!StrKey.isValidEd25519PublicKey(address)) {
    throw new Error("Invalid Stellar public key");
  }

  const keypair = Keypair.fromPublicKey(address);

  const messageBytes = Buffer.from(message);

  const signatureBytes = Buffer.from(
    signature,
    "base64"
  );

  return keypair.verify(
    messageBytes,
    signatureBytes
  );
}

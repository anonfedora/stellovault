import { Keypair } from "@stellar/stellar-sdk";
import { NotFoundError, ValidationError } from "../config/errors";
import { prisma } from "./database.service";

export class UserService {
    /**
     * Normalise a Stellar address to uppercase (consistent with auth.service.ts).
     */
    private normalise(address: string): string {
        return address.trim().toUpperCase();
    }

    /**
     * Validates that the provided string is a real Stellar public key.
     * Throws ValidationError if not.
     */
    private assertValidStellarAddress(address: string): void {
        try {
            Keypair.fromPublicKey(address);
        } catch {
            throw new ValidationError("Invalid Stellar wallet address");
        }
    }

    /**
     * Fetch a user by internal UUID, including their linked wallets.
     * The primary wallet is surfaced as a top-level field for convenience.
     *
     * @throws NotFoundError when the user does not exist.
     */
    async getUserById(id: string) {
        const user = await (prisma as any).user.findUnique({
            where: { id },
            include: {
                wallets: {
                    orderBy: [{ isPrimary: "desc" }, { createdAt: "asc" }],
                },
            },
        });

        if (!user) {
            throw new NotFoundError("User not found");
        }

        const { wallets, ...rest } = user;
        const primaryWallet = wallets.find((w: { isPrimary: boolean }) => w.isPrimary) ?? null;

        return { ...rest, primaryWallet, wallets };
    }

    /**
     * Idempotent upsert — if a user with this Stellar address already exists the
     * existing record is returned unchanged; otherwise a new user is created.
     *
     * @throws ValidationError when the address is not a valid Stellar public key.
     */
    async createUser(stellarAddress: string) {
        if (typeof stellarAddress !== "string" || stellarAddress.trim().length === 0) {
            throw new ValidationError("stellarAddress is required");
        }

        const normalised = this.normalise(stellarAddress);
        this.assertValidStellarAddress(normalised);

        return (prisma as any).user.upsert({
            where: { stellarAddress: normalised },
            update: {},
            create: { stellarAddress: normalised },
        });
    }
}

export default new UserService();

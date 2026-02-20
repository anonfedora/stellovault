import { PrismaClient } from "@prisma/client";

const prisma = new PrismaClient();

export class DatabaseService {
   async createUser(stellarAddress: string, options?: { label?: string; isPrimary?: boolean }) {
    // Create the user
    const user = await prisma.user.create({
        data: {
            name: null, // or you can pass a name if available
            role: "USER",
        },
    });

    //  Create the wallet linked to the user
    const wallet = await prisma.wallet.create({
        data: {
            userId: user.id,
            stellarAddress,
            isPrimary: options?.isPrimary ?? true, // default to primary
            label: options?.label ?? null,
            status: "ACTIVE",
        },
    });

    return { user, wallet };
}

    async getLoanById(id: string) {
        return prisma.loan.findUnique({
            where: { id },
            include: { borrower: true },
        });
    }

    async updateLoanStatus(id: string, status: any) {
        return prisma.loan.update({
            where: { id },
            data: { status },
        });
    }
}

export default new DatabaseService();
export { prisma };

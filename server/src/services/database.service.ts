import { PrismaClient } from "@prisma/client";

const prisma = new PrismaClient();

export class DatabaseService {
    async createUser(stellarAddress: string) {
        return prisma.user.create({
            data: { stellarAddress },
        });
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

import { NotFoundError, ValidationError } from "../config/errors";
import { prisma } from "./database.service";
import websocketService from "./websocket.service";

const VALID_VOTE_VALUES = new Set(["FOR", "AGAINST", "ABSTAIN"]);

interface CreateProposalRequest {
    title?: string;
    description?: string;
    type?: string;
    expiresAt?: string | Date;
}

interface CastVoteRequest {
    proposalId?: string;
    voterId?: string;
    vote?: string;
}

interface GovernanceProposal {
    id: string;
    title: string;
    description: string;
    type: string;
    status: string;
    expiresAt: Date;
    createdAt: Date;
    forVotes: number;
    againstVotes: number;
    abstainVotes: number;
    totalVotes: number;
}

function parseVote(value: string | undefined, fieldName: string): string {
    const vote = value?.trim().toUpperCase();
    if (!vote || !VALID_VOTE_VALUES.has(vote)) {
        throw new ValidationError(`${fieldName} must be one of: FOR, AGAINST, ABSTAIN`);
    }
    return vote;
}

function parseDate(value: string | Date | undefined, fieldName: string): Date {
    if (!value) {
        throw new ValidationError(`${fieldName} is required`);
    }

    const parsed = value instanceof Date ? value : new Date(value);
    if (Number.isNaN(parsed.getTime())) {
        throw new ValidationError(`${fieldName} must be a valid date`);
    }
    return parsed;
}

export class GovernanceService {
    async createProposal(payload: CreateProposalRequest): Promise<{ proposalId: string }> {
        const title = payload.title?.trim();
        const description = payload.description?.trim();
        const type = payload.type?.trim() || "GENERAL";
        
        if (!title) {
            throw new ValidationError("title is required");
        }
        if (!description) {
            throw new ValidationError("description is required");
        }

        const expiresAt = parseDate(payload.expiresAt, "expiresAt");
        if (expiresAt.getTime() <= Date.now()) {
            throw new ValidationError("expiresAt must be in the future");
        }

        const db: any = prisma;
        const proposal = await db.proposal.create({
            data: {
                title,
                description,
                type,
                status: "ACTIVE",
                expiresAt,
            },
        });

        return { proposalId: proposal.id };
    }

    async getProposal(proposalId: string): Promise<GovernanceProposal | null> {
        const db: any = prisma;
        const proposal = await db.proposal.findUnique({
            where: { id: proposalId },
            include: {
                votes: true,
            },
        });

        if (!proposal) {
            return null;
        }

        const forVotes = proposal.votes.filter((vote: { vote: string }) => vote.vote === "FOR").length;
        const againstVotes = proposal.votes.filter((vote: { vote: string }) => vote.vote === "AGAINST").length;
        const abstainVotes = proposal.votes.filter((vote: { vote: string }) => vote.vote === "ABSTAIN").length;
        const totalVotes = forVotes + againstVotes + abstainVotes;

        return {
            id: proposal.id,
            title: proposal.title,
            description: proposal.description,
            type: proposal.type,
            status: proposal.status,
            expiresAt: proposal.expiresAt,
            createdAt: proposal.createdAt,
            forVotes,
            againstVotes,
            abstainVotes,
            totalVotes,
        };
    }

    async listProposals(): Promise<GovernanceProposal[]> {
        const db: any = prisma;
        const proposals = await db.proposal.findMany({
            include: {
                votes: true,
            },
            orderBy: { createdAt: "desc" },
        });

        return proposals.map((proposal: any) => {
            const forVotes = proposal.votes.filter((vote: { vote: string }) => vote.vote === "FOR").length;
            const againstVotes = proposal.votes.filter((vote: { vote: string }) => vote.vote === "AGAINST").length;
            const abstainVotes = proposal.votes.filter((vote: { vote: string }) => vote.vote === "ABSTAIN").length;
            const totalVotes = forVotes + againstVotes + abstainVotes;

            return {
                id: proposal.id,
                title: proposal.title,
                description: proposal.description,
                type: proposal.type,
                status: proposal.status,
                expiresAt: proposal.expiresAt,
                createdAt: proposal.createdAt,
                forVotes,
                againstVotes,
                abstainVotes,
                totalVotes,
            };
        });
    }

    async castVote(payload: CastVoteRequest): Promise<void> {
        const proposalId = payload.proposalId?.trim();
        const voterId = payload.voterId?.trim();
        const vote = parseVote(payload.vote, "vote");

        if (!proposalId) {
            throw new ValidationError("proposalId is required");
        }
        if (!voterId) {
            throw new ValidationError("voterId is required");
        }

        const db: any = prisma;
        
        const proposal = await db.proposal.findUnique({
            where: { id: proposalId },
        });
        
        if (!proposal) {
            throw new NotFoundError("Proposal not found");
        }

        if (proposal.status !== "ACTIVE") {
            throw new ValidationError("Voting is not active for this proposal");
        }

        if (proposal.expiresAt.getTime() <= Date.now()) {
            throw new ValidationError("Voting period has expired for this proposal");
        }

        const existingVote = await db.vote.findFirst({
            where: {
                proposalId,
                voterId,
            },
        });

        if (existingVote) {
            await db.vote.update({
                where: { id: existingVote.id },
                data: { vote },
            });
        } else {
            await db.vote.create({
                data: {
                    proposalId,
                    voterId,
                    vote,
                },
            });
        }

        const updatedProposal = await this.getProposal(proposalId);
        if (updatedProposal) {
            const newTally = updatedProposal.totalVotes;
            websocketService.broadcastGovernanceVoteCast(proposalId, newTally);
        }
    }

    async getProposalVotes(proposalId: string): Promise<any[]> {
        const db: any = prisma;
        return db.vote.findMany({
            where: { proposalId },
            orderBy: { createdAt: "desc" },
        });
    }

    async getMetrics(): Promise<{ totalProposals: number; participationRate: number }> {
        const db: any = prisma;
        
        const [totalProposals, activeProposals, totalVotes] = await Promise.all([
            db.proposal.count(),
            db.proposal.count({ where: { status: "ACTIVE" } }),
            db.vote.count(),
        ]);

        const participationRate = totalProposals > 0 ? totalVotes / totalProposals : 0;

        return {
            totalProposals,
            participationRate,
        };
    }
}

export default new GovernanceService();

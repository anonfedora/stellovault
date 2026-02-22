import { PrismaClient, Prisma } from "@prisma/client";
import contractService from "./contract.service";
import eventMonitoringService from "./event-monitoring.service";

const prisma = new PrismaClient();

// Governance Status enum (matching schema.prisma)
export enum GovernanceStatus {
    OPEN = "OPEN",
    PASSED = "PASSED",
    REJECTED = "REJECTED",
    EXECUTED = "EXECUTED"
}

export interface CreateProposalRequest {
    title: string;
    description: string;
    proposerId: string;
    endsAt: Date;
    contractId?: string;
}

export interface CastVoteRequest {
    proposalId: string;
    voterAddress: string;
    vote: string; // "YES" | "NO" | "ABSTAIN"
    weight: string;
}

export interface ProposalFilters {
    status?: GovernanceStatus;
    proposerId?: string;
    limit?: number;
    offset?: number;
}

export class GovernanceService {
    /**
     * List all proposals with optional filtering and pagination.
     */
    async getProposals(filters?: ProposalFilters) {
        const where: any = {};
        
        if (filters?.status) {
            where.status = filters.status;
        }
        if (filters?.proposerId) {
            where.proposerId = filters.proposerId;
        }

        const [proposals, total] = await Promise.all([
            (prisma as any).governanceProposal.findMany({
                where,
                include: {
                    proposer: {
                        select: { id: true, stellarAddress: true, name: true }
                    },
                    votes: true
                },
                skip: filters?.offset,
                take: filters?.limit ?? 20,
                orderBy: { createdAt: "desc" }
            }),
            (prisma as any).governanceProposal.count({ where })
        ]);

        return { proposals, total };
    }

    /**
     * Create a new governance proposal.
     */
    async createProposal(req: CreateProposalRequest) {
        const proposal = await (prisma as any).governanceProposal.create({
            data: {
                title: req.title,
                description: req.description,
                proposerId: req.proposerId,
                endsAt: req.endsAt,
                status: GovernanceStatus.OPEN
            },
            include: {
                proposer: {
                    select: { id: true, stellarAddress: true, name: true }
                }
            }
        });

        return { proposal };
    }

    /**
     * Get a single proposal by ID.
     */
    async getProposalById(id: string) {
        const proposal = await (prisma as any).governanceProposal.findUnique({
            where: { id },
            include: {
                proposer: {
                    select: { id: true, stellarAddress: true, name: true }
                },
                votes: true
            }
        });

        if (!proposal) {
            throw new Error("Proposal not found");
        }

        return proposal;
    }

    /**
     * Get all votes for a proposal with voter addresses and weights.
     * Supports pagination to prevent unbounded results.
     */
    async getProposalVotes(proposalId: string, options?: { limit?: number; offset?: number }) {
        const proposal = await (prisma as any).governanceProposal.findUnique({
            where: { id: proposalId }
        });

        if (!proposal) {
            throw new Error("Proposal not found");
        }

        const limit = options?.limit ?? 100;
        const offset = options?.offset ?? 0;

        const [votes, total] = await Promise.all([
            (prisma as any).governanceVote.findMany({
                where: { proposalId },
                orderBy: { createdAt: "desc" },
                skip: offset,
                take: limit
            }),
            (prisma as any).governanceVote.count({ where: { proposalId } })
        ]);

        return { votes, total, limit, offset };
    }

    /**
     * Submit a vote on a proposal.
     */
    async castVote(req: CastVoteRequest) {
        const proposal = await (prisma as any).governanceProposal.findUnique({
            where: { id: req.proposalId }
        });

        if (!proposal) {
            throw new Error("Proposal not found");
        }

        if (proposal.status !== GovernanceStatus.OPEN) {
            throw new Error(`Cannot vote on proposal with status: ${proposal.status}`);
        }

        if (new Date() > proposal.endsAt) {
            throw new Error("Voting period has ended");
        }

        // Check for existing vote
        const existingVote = await (prisma as any).governanceVote.findFirst({
            where: {
                proposalId: req.proposalId,
                voterAddress: req.voterAddress
            }
        });

        if (existingVote) {
            throw new Error("User has already voted on this proposal");
        }

        // Create vote
        const vote = await (prisma as any).governanceVote.create({
            data: {
                proposalId: req.proposalId,
                voterAddress: req.voterAddress,
                vote: req.vote,
                weight: new Prisma.Decimal(req.weight)
            }
        });

        return { vote };
    }

    /**
     * Get protocol governance health metrics.
     */
    async getMetrics() {
        const [totalProposals, openProposals, passedProposals, rejectedProposals, executedProposals, totalVotes] = await Promise.all([
            (prisma as any).governanceProposal.count(),
            (prisma as any).governanceProposal.count({ where: { status: GovernanceStatus.OPEN } }),
            (prisma as any).governanceProposal.count({ where: { status: GovernanceStatus.PASSED } }),
            (prisma as any).governanceProposal.count({ where: { status: GovernanceStatus.REJECTED } }),
            (prisma as any).governanceProposal.count({ where: { status: GovernanceStatus.EXECUTED } }),
            (prisma as any).governanceVote.count()
        ]);

        const avgWeightResult = await (prisma as any).governanceVote.aggregate({
            _avg: { weight: true }
        });

        const avgWeight = avgWeightResult._avg.weight ?? new Prisma.Decimal(0);

        // Calculate participation rate (votes per proposal)
        const participationRate = totalProposals > 0 ? totalVotes / totalProposals : 0;

        return {
            proposals: {
                total: totalProposals,
                open: openProposals,
                passed: passedProposals,
                rejected: rejectedProposals,
                executed: executedProposals
            },
            voting: {
                totalVotes,
                avgVoteWeight: avgWeight.toString(),
                participationRate: Number(participationRate.toFixed(2))
            }
        };
    }
}

export default new GovernanceService();

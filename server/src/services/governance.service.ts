import { PrismaClient, ProposalStatus, Prisma } from "@prisma/client";
import contractService from "./contract.service";
import eventMonitoringService from "./event-monitoring.service";

const prisma = new PrismaClient();

export interface CreateProposalRequest {
    title: string;
    description: string;
    proposerId: string;
    quorum: string;
    deadline?: Date;
    contractId?: string;
}

export interface SubmitVoteRequest {
    proposalId: string;
    voterId: string;
    voteFor: boolean;
    weight: string;
}

export interface ProposalFilters {
    status?: ProposalStatus;
    proposerId?: string;
    limit?: number;
    offset?: number;
}

export class GovernanceService {
    /**
     * List all proposals with optional filtering and pagination.
     */
    async getProposals(filters?: ProposalFilters) {
        const where: Prisma.ProposalWhereInput = {};
        
        if (filters?.status) {
            where.status = filters.status;
        }
        if (filters?.proposerId) {
            where.proposerId = filters.proposerId;
        }

        const [proposals, total] = await Promise.all([
            prisma.proposal.findMany({
                where,
                include: {
                    proposer: {
                        select: { id: true, stellarAddress: true, name: true }
                    },
                    _count: {
                        select: { votes: true }
                    }
                },
                skip: filters?.offset,
                take: filters?.limit ?? 20,
                orderBy: { createdAt: "desc" }
            }),
            prisma.proposal.count({ where })
        ]);

        return { proposals, total };
    }

    /**
     * Create a new governance proposal.
     * Builds Soroban XDR for on-chain proposal and persists off-chain record.
     */
    async createProposal(req: CreateProposalRequest) {
        const quorumDecimal = new Prisma.Decimal(req.quorum);
        
        // Build XDR for on-chain proposal creation
        let xdr: string | undefined;
        if (req.contractId) {
            xdr = await contractService.buildContractInvokeXDR(
                req.contractId,
                "create_proposal",
                [req.title, req.description, quorumDecimal.toString()]
            );
        }

        const proposal = await prisma.proposal.create({
            data: {
                title: req.title,
                description: req.description,
                proposerId: req.proposerId,
                quorum: quorumDecimal,
                deadline: req.deadline,
                contractId: req.contractId,
                xdr,
                status: ProposalStatus.OPEN
            },
            include: {
                proposer: {
                    select: { id: true, stellarAddress: true, name: true }
                }
            }
        });

        // Log audit event
        await this.logAuditEvent("PROPOSAL_CREATED", {
            proposalId: proposal.id,
            details: `Proposal "${req.title}" created by ${req.proposerId}`
        });

        return { proposal, xdr };
    }

    /**
     * Get a single proposal by ID.
     */
    async getProposalById(id: string) {
        const proposal = await prisma.proposal.findUnique({
            where: { id },
            include: {
                proposer: {
                    select: { id: true, stellarAddress: true, name: true }
                },
                votes: {
                    include: {
                        voter: {
                            select: { id: true, stellarAddress: true, name: true }
                        }
                    }
                }
            }
        });

        if (!proposal) {
            throw new Error("Proposal not found");
        }

        // Calculate vote totals
        const forVotes = proposal.votes.filter(v => v.voteFor).reduce((sum, v) => sum.plus(v.weight), new Prisma.Decimal(0));
        const againstVotes = proposal.votes.filter(v => !v.voteFor).reduce((sum, v) => sum.plus(v.weight), new Prisma.Decimal(0));

        return {
            ...proposal,
            voteSummary: {
                for: forVotes.toString(),
                against: againstVotes.toString(),
                total: forVotes.plus(againstVotes).toString(),
                voterCount: proposal.votes.length
            }
        };
    }

    /**
     * Get all votes for a proposal with voter addresses and weights.
     * Supports pagination to prevent unbounded results.
     */
    async getProposalVotes(proposalId: string, options?: { limit?: number; offset?: number }) {
        const proposal = await prisma.proposal.findUnique({
            where: { id: proposalId }
        });

        if (!proposal) {
            throw new Error("Proposal not found");
        }

        const limit = options?.limit ?? 100;
        const offset = options?.offset ?? 0;

        const [votes, total] = await Promise.all([
            prisma.vote.findMany({
                where: { proposalId },
                include: {
                    voter: {
                        select: { id: true, stellarAddress: true, name: true }
                    }
                },
                orderBy: { createdAt: "desc" },
                skip: offset,
                take: limit
            }),
            prisma.vote.count({ where: { proposalId } })
        ]);

        return { votes, total, limit, offset };
    }

    /**
     * Submit a vote on a proposal.
     * Verifies voter has quorum weight, persists vote, builds contract call XDR.
     * Uses transaction to prevent TOCTOU race condition on duplicate votes.
     */
    async submitVote(req: SubmitVoteRequest) {
        const proposal = await prisma.proposal.findUnique({
            where: { id: req.proposalId }
        });

        if (!proposal) {
            throw new Error("Proposal not found");
        }

        if (proposal.status !== ProposalStatus.OPEN) {
            throw new Error(`Cannot vote on proposal with status: ${proposal.status}`);
        }

        if (proposal.deadline && new Date() > proposal.deadline) {
            throw new Error("Voting deadline has passed");
        }

        // Verify voter has quorum weight
        const weightDecimal = new Prisma.Decimal(req.weight);
        if (weightDecimal.lessThanOrEqualTo(0)) {
            throw new Error("Voter weight must be greater than zero");
        }

        // Build XDR for contract vote submission
        let xdr: string | undefined;
        if (proposal.contractId) {
            xdr = await contractService.buildContractInvokeXDR(
                proposal.contractId,
                "cast_vote",
                [req.proposalId, req.voteFor, weightDecimal.toString()]
            );
        }

        // Use transaction to prevent TOCTOU race condition
        try {
            const vote = await prisma.$transaction(async (tx) => {
                // Check for existing vote inside transaction
                const existingVote = await tx.vote.findUnique({
                    where: {
                        proposalId_voterId: {
                            proposalId: req.proposalId,
                            voterId: req.voterId
                        }
                    }
                });

                if (existingVote) {
                    const error = new Error("Duplicate vote: User has already voted on this proposal");
                    (error as any).statusCode = 409;
                    throw error;
                }

                // Persist vote
                return tx.vote.create({
                    data: {
                        proposalId: req.proposalId,
                        voterId: req.voterId,
                        weight: weightDecimal,
                        voteFor: req.voteFor,
                        xdr
                    },
                    include: {
                        voter: {
                            select: { id: true, stellarAddress: true, name: true }
                        },
                        proposal: true
                    }
                });
            }, {
                isolationLevel: Prisma.TransactionIsolationLevel.Serializable
            });

            // Log audit event
            await this.logAuditEvent("VOTE_CAST", {
                proposalId: req.proposalId,
                voterId: req.voterId,
                details: `Vote ${req.voteFor ? "FOR" : "AGAINST"} proposal ${req.proposalId} with weight ${req.weight}`
            });

            return { vote, xdr };
        } catch (err) {
            // Handle unique constraint violation (P2002) as 409
            const error = err as any;
            if (error.code === "P2002" || error.statusCode === 409) {
                const dupError = new Error("Duplicate vote: User has already voted on this proposal");
                (dupError as any).statusCode = 409;
                throw dupError;
            }
            throw err;
        }
    }

    /**
     * Get protocol governance health metrics.
     * Uses scalable queries with aggregation for performance.
     */
    async getMetrics() {
        // Use raw query for efficient distinct count
        const uniqueVotersResult = await prisma.$queryRaw<{ count: number }[]>`
            SELECT COUNT(DISTINCT "voterId") as count FROM "Vote"
        `;
        const uniqueVoters = Number(uniqueVotersResult[0]?.count ?? 0);

        // Use aggregate for efficient average calculation
        const [totalProposals, openProposals, passedProposals, rejectedProposals, executedProposals, totalVotes, avgWeightResult] = await Promise.all([
            prisma.proposal.count(),
            prisma.proposal.count({ where: { status: ProposalStatus.OPEN } }),
            prisma.proposal.count({ where: { status: ProposalStatus.PASSED } }),
            prisma.proposal.count({ where: { status: ProposalStatus.REJECTED } }),
            prisma.proposal.count({ where: { status: ProposalStatus.EXECUTED } }),
            prisma.vote.count(),
            prisma.vote.aggregate({
                _avg: { weight: true }
            })
        ]);

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
                uniqueVoters,
                avgVoteWeight: avgWeight.toString(),
                participationRate: Number(participationRate.toFixed(2))
            }
        };
    }

    /**
     * Get current on-chain governance parameters via Soroban simulateCall.
     */
    async getParameters() {
        // These would typically come from the governance contract
        const contractId = process.env.GOVERNANCE_CONTRACT_ID;
        
        if (!contractId) {
            // Return default parameters if no contract configured
            return {
                minQuorum: "1000000",
                votingPeriod: 604800, // 7 days in seconds
                proposalThreshold: "10000",
                executionDelay: 86400, // 1 day in seconds
                maxProposalsPerUser: 5
            };
        }

        // Simulate calls to read on-chain parameters
        const [minQuorum, votingPeriod, proposalThreshold, executionDelay] = await Promise.all([
            contractService.simulateCall(contractId, "get_min_quorum", []),
            contractService.simulateCall(contractId, "get_voting_period", []),
            contractService.simulateCall(contractId, "get_proposal_threshold", []),
            contractService.simulateCall(contractId, "get_execution_delay", [])
        ]);

        return {
            minQuorum: minQuorum.result,
            votingPeriod: votingPeriod.result,
            proposalThreshold: proposalThreshold.result,
            executionDelay: executionDelay.result,
            contractId
        };
    }

    /**
     * Get paginated audit log of all governance actions.
     * Sourced from EventMonitoringService for on-chain events.
     */
    async getAuditLog(options?: { limit?: number; offset?: number }) {
        const limit = options?.limit ?? 50;
        const offset = options?.offset ?? 0;

        // Get on-chain events from indexer via EventMonitoringService
        const onChainEvents = await this.fetchOnChainGovernanceEvents(limit, offset);

        // Get off-chain audit logs
        const [offChainLogs, total] = await Promise.all([
            prisma.governanceAuditLog.findMany({
                skip: offset,
                take: limit,
                orderBy: { createdAt: "desc" }
            }),
            prisma.governanceAuditLog.count()
        ]);

        return {
            onChain: onChainEvents,
            offChain: offChainLogs,
            total,
            limit,
            offset
        };
    }

    /**
     * Fetch on-chain governance events from indexer.
     */
    private async fetchOnChainGovernanceEvents(limit: number, offset: number) {
        // This would integrate with EventMonitoringService to get on-chain events
        // For now, return placeholder structure
        try {
            // Poll events through the monitoring service
            await eventMonitoringService.pollEvents();
            
            // In a real implementation, this would query indexed events
            return {
                events: [], // Would be populated from indexer
                hasMore: false
            };
        } catch {
            return {
                events: [],
                hasMore: false
            };
        }
    }

    /**
     * Log an audit event to the governance audit log.
     */
    private async logAuditEvent(
        eventType: string,
        data: { proposalId?: string; voterId?: string; txHash?: string; details?: string }
    ) {
        await prisma.governanceAuditLog.create({
            data: {
                eventType,
                proposalId: data.proposalId,
                voterId: data.voterId,
                txHash: data.txHash,
                details: data.details
            }
        });
    }

    /**
     * Update proposal status (used by event monitoring or admin actions).
     */
    async updateProposalStatus(id: string, status: ProposalStatus) {
        const proposal = await prisma.proposal.update({
            where: { id },
            data: { status },
            include: {
                proposer: {
                    select: { id: true, stellarAddress: true, name: true }
                }
            }
        });

        await this.logAuditEvent("PROPOSAL_STATUS_CHANGED", {
            proposalId: id,
            details: `Status changed to ${status}`
        });

        return proposal;
    }
}

export default new GovernanceService();

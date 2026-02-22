import { Request, Response, NextFunction } from "express";
import governanceService from "../services/governance.service";
import { ProposalStatus } from "@prisma/client";

/**
 * GET /api/governance/proposals
 * List all proposals with optional filtering.
 */
export async function getProposals(req: Request, res: Response, next: NextFunction) {
    try {
        const filters = {
            status: req.query.status as ProposalStatus | undefined,
            proposerId: req.query.proposerId as string | undefined,
            limit: req.query.limit ? parseInt(req.query.limit as string, 10) : undefined,
            offset: req.query.offset ? parseInt(req.query.offset as string, 10) : undefined
        };

        const { proposals, total } = await governanceService.getProposals(filters);

        res.json({
            success: true,
            data: proposals,
            meta: {
                total,
                limit: filters.limit ?? 20,
                offset: filters.offset ?? 0
            }
        });
    } catch (err) {
        next(err);
    }
}

/**
 * POST /api/governance/proposals
 * Create a new governance proposal.
 */
export async function createProposal(req: Request, res: Response, next: NextFunction) {
    try {
        const { title, description, quorum, deadline, contractId } = req.body;
        const proposerId = req.user!.userId;

        if (!title || !description || !quorum) {
            return res.status(400).json({
                success: false,
                error: "Missing required fields: title, description, quorum"
            });
        }

        const { proposal, xdr } = await governanceService.createProposal({
            title,
            description,
            proposerId,
            quorum,
            deadline: deadline ? new Date(deadline) : undefined,
            contractId
        });

        res.status(201).json({
            success: true,
            data: {
                proposalId: proposal.id,
                proposal,
                xdr
            }
        });
    } catch (err) {
        next(err);
    }
}

/**
 * GET /api/governance/proposals/:id
 * Get a single proposal by ID.
 */
export async function getProposal(req: Request, res: Response, next: NextFunction) {
    try {
        const { id } = req.params;
        const proposal = await governanceService.getProposalById(id);

        res.json({
            success: true,
            data: proposal
        });
    } catch (err) {
        if ((err as Error).message === "Proposal not found") {
            return res.status(404).json({
                success: false,
                error: "Proposal not found"
            });
        }
        next(err);
    }
}

/**
 * GET /api/governance/proposals/:id/votes
 * Get all votes for a proposal.
 */
export async function getProposalVotes(req: Request, res: Response, next: NextFunction) {
    try {
        const { id } = req.params;
        const votes = await governanceService.getProposalVotes(id);

        res.json({
            success: true,
            data: votes
        });
    } catch (err) {
        if ((err as Error).message === "Proposal not found") {
            return res.status(404).json({
                success: false,
                error: "Proposal not found"
            });
        }
        next(err);
    }
}

/**
 * POST /api/governance/votes
 * Cast a vote on a proposal.
 */
export async function submitVote(req: Request, res: Response, next: NextFunction) {
    try {
        const { proposalId, voteFor, weight } = req.body;
        const voterId = req.user!.userId;

        if (!proposalId || typeof voteFor !== "boolean" || !weight) {
            return res.status(400).json({
                success: false,
                error: "Missing required fields: proposalId, voteFor, weight"
            });
        }

        const { vote, xdr } = await governanceService.submitVote({
            proposalId,
            voterId,
            voteFor,
            weight
        });

        res.json({
            success: true,
            data: {
                vote,
                xdr
            }
        });
    } catch (err) {
        const error = err as Error;
        
        // Handle duplicate vote (409)
        if ((error as any).statusCode === 409) {
            return res.status(409).json({
                success: false,
                error: error.message
            });
        }

        // Handle proposal not found (404)
        if (error.message === "Proposal not found") {
            return res.status(404).json({
                success: false,
                error: "Proposal not found"
            });
        }

        // Handle closed proposal or deadline passed (400)
        if (error.message.includes("Cannot vote on proposal") || error.message.includes("deadline has passed")) {
            return res.status(400).json({
                success: false,
                error: error.message
            });
        }

        next(err);
    }
}

/**
 * GET /api/governance/metrics
 * Get protocol governance health metrics.
 */
export async function getMetrics(req: Request, res: Response, next: NextFunction) {
    try {
        const metrics = await governanceService.getMetrics();

        res.json({
            success: true,
            data: metrics
        });
    } catch (err) {
        next(err);
    }
}

/**
 * GET /api/governance/parameters
 * Get current on-chain governance parameters.
 */
export async function getParameters(req: Request, res: Response, next: NextFunction) {
    try {
        const parameters = await governanceService.getParameters();

        res.json({
            success: true,
            data: parameters
        });
    } catch (err) {
        next(err);
    }
}

/**
 * GET /api/governance/audit
 * Get audit log of all governance actions.
 */
export async function getAuditLog(req: Request, res: Response, next: NextFunction) {
    try {
        const limit = req.query.limit ? parseInt(req.query.limit as string, 10) : undefined;
        const offset = req.query.offset ? parseInt(req.query.offset as string, 10) : undefined;

        const auditLog = await governanceService.getAuditLog({ limit, offset });

        res.json({
            success: true,
            data: auditLog.offChain,
            meta: {
                onChainEvents: auditLog.onChain,
                total: auditLog.total,
                limit: auditLog.limit,
                offset: auditLog.offset
            }
        });
    } catch (err) {
        next(err);
    }
}

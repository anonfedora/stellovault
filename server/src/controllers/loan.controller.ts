import { Request, Response, NextFunction } from "express";
import { UnauthorizedError } from "../config/errors";
import loanService from "../services/loan.service";

export async function listLoans(req: Request, res: Response, next: NextFunction) {
    try {
        const { borrowerId, lenderId, status } = req.query;
        const loans = await loanService.listLoans(
            typeof borrowerId === "string" ? borrowerId : undefined,
            typeof lenderId === "string" ? lenderId : undefined,
            typeof status === "string" ? status : undefined
        );
        res.json({ success: true, data: loans });
    } catch (err) { next(err); }
}

export async function getLoan(req: Request, res: Response, next: NextFunction) {
    try {
        const loan = await loanService.getLoan(req.params.id);
        res.json({ success: true, data: loan });
    } catch (err) { next(err); }
}

export async function createLoan(req: Request, res: Response, next: NextFunction) {
    try {
        const requestingUserId = req.user?.userId;
        if (!requestingUserId) {
            throw new UnauthorizedError("Authentication required");
        }

        const result = await loanService.issueLoan({
            ...req.body,
            requestingUserId,
        });
        res.status(201).json({
            success: true,
            data: {
                loanId: result.loanId,
                xdr: result.xdr,
                loan: result.loan,
            },
        });
    } catch (err) { next(err); }
}

export async function recordRepayment(req: Request, res: Response, next: NextFunction) {
    try {
        const requestingUserId = req.user?.userId;
        if (!requestingUserId) {
            throw new UnauthorizedError("Authentication required");
        }

        const result = await loanService.recordRepayment({
            ...req.body,
            requestingUserId,
        });
        res.json({ success: true, data: result });
    } catch (err) { next(err); }
}

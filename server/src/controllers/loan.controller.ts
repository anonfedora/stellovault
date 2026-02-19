import { Request, Response, NextFunction } from "express";

export async function listLoans(req: Request, res: Response, next: NextFunction) {
    try {
        // query: borrowerId, lenderId, status
        res.json({ success: true, data: [] });
    } catch (err) { next(err); }
}

export async function getLoan(req: Request, res: Response, next: NextFunction) {
    try {
        res.json({ success: true, data: null });
    } catch (err) { next(err); }
}

export async function createLoan(req: Request, res: Response, next: NextFunction) {
    try {
        // Returns unsigned XDR for Soroban contract invocation
        res.status(201).json({ success: true, data: { loanId: "TODO", xdr: "BASE64_XDR" } });
    } catch (err) { next(err); }
}

export async function recordRepayment(req: Request, res: Response, next: NextFunction) {
    try {
        res.json({ success: true, data: null });
    } catch (err) { next(err); }
}

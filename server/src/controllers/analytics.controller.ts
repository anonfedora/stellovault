import { Request, Response, NextFunction } from "express";
import analyticsService from "../services/analytics.service";

export async function getPlatformStats(_req: Request, res: Response, next: NextFunction) {
    try {
        const stats = await analyticsService.getPlatformStats();
        // Acceptance requires flat JSON object with all stats
        res.json(stats);
    } catch (err) { next(err); }
}

export async function getProtocolAnalytics(_req: Request, res: Response, next: NextFunction) {
    try {
        const data = await analyticsService.getProtocolAnalytics();
        res.json(data);
    } catch (err) { next(err); }
}

export async function getDashboardMetrics(_req: Request, res: Response, next: NextFunction) {
    try {
        const metrics = await analyticsService.calculateDashboardMetrics();
        res.json(metrics);
    } catch (err) { next(err); }
}

export async function getVolumeReport(req: Request, res: Response, next: NextFunction) {
    try {
        const period = req.query.period as 'hour' | 'day' | 'week' | 'month' || 'day';
        const report = await analyticsService.generateVolumeReport(period);
        res.json(report);
    } catch (err) { next(err); }
}

export async function getUserAnalytics(req: Request, res: Response, next: NextFunction) {
    try {
        const userId = req.query.userId as string;
        const analytics = await analyticsService.analyzeUserBehavior(userId);
        res.json(analytics);
    } catch (err) { next(err); }
}

export async function getPlatformPerformance(_req: Request, res: Response, next: NextFunction) {
    try {
        const performance = await analyticsService.trackPlatformPerformance();
        res.json(performance);
    } catch (err) { next(err); }
}

export async function generateCustomReport(req: Request, res: Response, next: NextFunction) {
    try {
        const reportRequest = req.body;
        const report = await analyticsService.generateCustomReport(reportRequest);
        res.json(report);
    } catch (err) { next(err); }
}

export async function getComplianceMetrics(req: Request, res: Response, next: NextFunction) {
    try {
        const type = req.query.type as 'kyc' | 'aml' | 'audit' | 'retention' || 'kyc';
        const metrics = await analyticsService.generateComplianceReport(type);
        res.json(metrics);
    } catch (err) { next(err); }
}

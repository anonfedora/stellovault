import { Router } from "express";
import eventMonitoringService from "../services/event-monitoring.service";
import { authMiddleware } from "../middleware/auth.middleware";
import { requirePermission } from "../middleware/rbac.middleware";

const router = Router();

// ─────────────────────────────────────────────
// Public Routes (Health Check)
// ─────────────────────────────────────────────

/**
 * GET /api/monitoring/health
 * Basic health check endpoint
 */
router.get("/health", async (req, res) => {
  try {
    const dashboardData = await eventMonitoringService.getDashboardData();
    res.json({
      status: "healthy",
      isRunning: dashboardData.isRunning,
      lastLedger: dashboardData.lastLedger,
      eventQueueSize: dashboardData.eventQueueSize,
    });
  } catch (error) {
    res.status(500).json({ error: "Health check failed" });
  }
});

// ─────────────────────────────────────────────
// Protected Routes (Require Authentication)
// ─────────────────────────────────────────────

router.use(authMiddleware);

/**
 * GET /api/monitoring/dashboard
 * Get comprehensive dashboard data
 */
router.get("/dashboard", requirePermission("monitoring:read"), async (req, res) => {
  try {
    const dashboardData = await eventMonitoringService.getDashboardData();
    res.json(dashboardData);
  } catch (error) {
    res.status(500).json({ error: "Failed to fetch dashboard data" });
  }
});

/**
 * GET /api/monitoring/events
 * Get event logs with optional filtering
 */
router.get("/events", requirePermission("monitoring:read"), async (req, res) => {
  try {
    const { eventType, startTime, endTime } = req.query;

    const filter: any = {};
    if (eventType) {
      filter.eventType = Array.isArray(eventType) ? eventType : [eventType];
    }
    if (startTime && endTime) {
      filter.timeRange = {
        start: new Date(startTime as string),
        end: new Date(endTime as string),
      };
    }

    const events = await eventMonitoringService.getEventLogs(filter);
    res.json(events);
  } catch (error) {
    res.status(500).json({ error: "Failed to fetch event logs" });
  }
});

/**
 * GET /api/monitoring/health-metrics
 * Get health metrics with optional filtering
 */
router.get("/health-metrics", requirePermission("monitoring:read"), async (req, res) => {
  try {
    const { metricName, startTime, endTime } = req.query;

    const timeRange =
      startTime && endTime
        ? {
            start: new Date(startTime as string),
            end: new Date(endTime as string),
          }
        : undefined;

    const metrics = await eventMonitoringService.getHealthMetrics(
      metricName as string | undefined,
      timeRange,
    );
    res.json(metrics);
  } catch (error) {
    res.status(500).json({ error: "Failed to fetch health metrics" });
  }
});

/**
 * GET /api/monitoring/transaction-stats
 * Get transaction statistics for a time range
 */
router.get("/transaction-stats", requirePermission("monitoring:read"), async (req, res) => {
  try {
    const { startTime, endTime } = req.query;

    if (!startTime || !endTime) {
      return res.status(400).json({ error: "startTime and endTime are required" });
    }

    const stats = await eventMonitoringService.getTransactionStats({
      start: new Date(startTime as string),
      end: new Date(endTime as string),
    });
    res.json(stats);
  } catch (error) {
    res.status(500).json({ error: "Failed to fetch transaction stats" });
  }
});

/**
 * GET /api/monitoring/contract-events/:contractId
 * Get contract events for a specific contract
 */
router.get(
  "/contract-events/:contractId",
  requirePermission("monitoring:read"),
  async (req, res) => {
    try {
      const { contractId } = req.params;
      const { startTime, endTime } = req.query;

      const timeRange =
        startTime && endTime
          ? {
              start: new Date(startTime as string),
              end: new Date(endTime as string),
            }
          : undefined;

      const stats = await eventMonitoringService.getContractEventStats(
        contractId,
        timeRange,
      );
      res.json(stats);
    } catch (error) {
      res.status(500).json({ error: "Failed to fetch contract events" });
    }
  },
);

/**
 * GET /api/monitoring/soroban-events/:contractId
 * Get Soroban contract events from RPC
 */
router.get(
  "/soroban-events/:contractId",
  requirePermission("monitoring:read"),
  async (req, res) => {
    try {
      const { contractId } = req.params;
      const { minLedger, maxLedger } = req.query;

      const filter: any = {};
      if (minLedger) filter.minLedger = parseInt(minLedger as string);
      if (maxLedger) filter.maxLedger = parseInt(maxLedger as string);

      const events = await eventMonitoringService.getContractEvents(contractId, filter);
      res.json(events);
    } catch (error) {
      res.status(500).json({ error: "Failed to fetch Soroban events" });
    }
  },
);

// ─────────────────────────────────────────────
// Admin Routes (Require Admin Permission)
// ─────────────────────────────────────────────

/**
 * POST /api/monitoring/start
 * Start the event monitoring service
 */
router.post("/start", requirePermission("monitoring:write"), async (req, res) => {
  try {
    await eventMonitoringService.start();
    res.json({ message: "Event monitoring service started" });
  } catch (error) {
    res.status(500).json({ error: "Failed to start monitoring service" });
  }
});

/**
 * POST /api/monitoring/stop
 * Stop the event monitoring service
 */
router.post("/stop", requirePermission("monitoring:write"), async (req, res) => {
  try {
    await eventMonitoringService.stop();
    res.json({ message: "Event monitoring service stopped" });
  } catch (error) {
    res.status(500).json({ error: "Failed to stop monitoring service" });
  }
});

/**
 * POST /api/monitoring/poll
 * Manually trigger event polling
 */
router.post("/poll", requirePermission("monitoring:write"), async (req, res) => {
  try {
    await eventMonitoringService.pollEvents();
    res.json({ message: "Event polling triggered" });
  } catch (error) {
    res.status(500).json({ error: "Failed to poll events" });
  }
});

/**
 * POST /api/monitoring/collect-metrics
 * Manually trigger health metrics collection
 */
router.post("/collect-metrics", requirePermission("monitoring:write"), async (req, res) => {
  try {
    await eventMonitoringService.collectHealthMetrics();
    res.json({ message: "Health metrics collection triggered" });
  } catch (error) {
    res.status(500).json({ error: "Failed to collect health metrics" });
  }
});

export default router;

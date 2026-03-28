import { Queue, Worker, Job, QueueEvents } from "bullmq";
import { Redis } from "ioredis";
import { env } from "../config/env";
import { PaymentStatus, TransactionStatus } from "../types/loan";
import contractService from "./contract.service";

const REDIS_HOST = process.env.REDIS_HOST || "localhost";
const REDIS_PORT = parseInt(process.env.REDIS_PORT || "6379", 10);
const MAX_RETRIES = 5;
const INITIAL_BACKOFF_MS = 1000;

interface TransactionJobData {
    idempotencyKey: string;
    contractId: string;
    method: string;
    args: any[];
    sourcePublicKey: string;
    userSignedXdr?: string;
    metadata?: Record<string, any>;
}

interface TransactionResult {
    success: boolean;
    txHash?: string;
    error?: string;
    attempts: number;
}

export class TransactionQueueService {
    private queue: Queue<TransactionJobData, TransactionResult>;
    private worker: Worker<TransactionJobData, TransactionResult>;
    private dlq: Queue<TransactionJobData, TransactionResult>;
    private queueEvents: QueueEvents;
    private connection: Redis;

    constructor() {
        this.connection = new Redis({
            host: REDIS_HOST,
            port: REDIS_PORT,
            maxRetriesPerRequest: null,
        });

        // Main transaction queue
        this.queue = new Queue<TransactionJobData, TransactionResult>("stellar-transactions", {
            connection: this.connection,
            defaultJobOptions: {
                attempts: MAX_RETRIES,
                backoff: {
                    type: "exponential",
                    delay: INITIAL_BACKOFF_MS,
                },
                removeOnComplete: {
                    count: 1000,
                    age: 24 * 3600, // 24 hours
                },
                removeOnFail: false,
            },
        });

        // Dead Letter Queue for failed transactions
        this.dlq = new Queue<TransactionJobData, TransactionResult>("stellar-transactions-dlq", {
            connection: this.connection,
        });

        // Queue events for monitoring
        this.queueEvents = new QueueEvents("stellar-transactions", {
            connection: this.connection,
        });

        // Worker to process transactions
        this.worker = new Worker<TransactionJobData, TransactionResult>(
            "stellar-transactions",
            async (job: Job<TransactionJobData, TransactionResult>) => {
                return await this.processTransaction(job);
            },
            {
                connection: this.connection,
                concurrency: 10, // Process up to 10 transactions concurrently
                limiter: {
                    max: 100,
                    duration: 1000, // Max 100 transactions per second
                },
            }
        );

        this.setupEventListeners();
    }

    private setupEventListeners(): void {
        this.worker.on("completed", (job: Job<TransactionJobData, TransactionResult>) => {
            console.log(`✓ Transaction job ${job.id} completed:`, job.returnvalue);
        });

        this.worker.on("failed", async (job: Job<TransactionJobData, TransactionResult> | undefined, err: Error) => {
            if (!job) return;

            console.error(`✗ Transaction job ${job.id} failed after ${job.attemptsMade} attempts:`, err.message);

            // Move to DLQ if max retries exceeded
            if (job.attemptsMade >= MAX_RETRIES) {
                await this.moveToDLQ(job, err);
            }
        });

        this.worker.on("error", (err: Error) => {
            console.error("Worker error:", err);
        });

        this.queueEvents.on("failed", ({ jobId, failedReason }) => {
            console.error(`Job ${jobId} failed with reason: ${failedReason}`);
        });
    }

    private async processTransaction(
        job: Job<TransactionJobData, TransactionResult>
    ): Promise<TransactionResult> {
        const { idempotencyKey, contractId, method, args, sourcePublicKey, userSignedXdr, metadata } = job.data;

        console.log(`Processing transaction job ${job.id} (attempt ${job.attemptsMade + 1}/${MAX_RETRIES})`);
        console.log(`Idempotency key: ${idempotencyKey}`);

        try {
            // Check if transaction already processed (idempotency)
            const existingResult = await this.checkIdempotency(idempotencyKey);
            if (existingResult) {
                console.log(`Transaction ${idempotencyKey} already processed, returning cached result`);
                return existingResult;
            }

            let txHash: string;

            if (userSignedXdr) {
                // Submit pre-signed transaction
                txHash = await contractService.submitSignedTransaction(userSignedXdr);
            } else {
                // Build, sign, and submit transaction
                const xdr = await contractService.buildContractInvokeXDR(
                    contractId,
                    method,
                    args,
                    sourcePublicKey
                );
                txHash = await contractService.submitSignedTransaction(xdr);
            }

            const result: TransactionResult = {
                success: true,
                txHash,
                attempts: job.attemptsMade + 1,
            };

            // Store result for idempotency
            await this.storeIdempotencyResult(idempotencyKey, result);

            return result;
        } catch (error: any) {
            const errorMessage = error.message || String(error);

            // Check if error is retryable
            if (this.isRetryableError(errorMessage)) {
                console.log(`Retryable error for job ${job.id}: ${errorMessage}`);
                throw error; // Let BullMQ retry
            }

            // Non-retryable error - fail immediately
            console.error(`Non-retryable error for job ${job.id}: ${errorMessage}`);
            const result: TransactionResult = {
                success: false,
                error: errorMessage,
                attempts: job.attemptsMade + 1,
            };

            await this.storeIdempotencyResult(idempotencyKey, result);
            throw new Error(errorMessage);
        }
    }

    private isRetryableError(errorMessage: string): boolean {
        const retryablePatterns = [
            "timeout",
            "network",
            "connection",
            "ECONNREFUSED",
            "ETIMEDOUT",
            "sequence",
            "tx_bad_seq",
            "congestion",
            "rate limit",
        ];

        return retryablePatterns.some((pattern) =>
            errorMessage.toLowerCase().includes(pattern.toLowerCase())
        );
    }

    private async checkIdempotency(key: string): Promise<TransactionResult | null> {
        const cached = await this.connection.get(`idempotency:${key}`);
        return cached ? JSON.parse(cached) : null;
    }

    private async storeIdempotencyResult(key: string, result: TransactionResult): Promise<void> {
        await this.connection.setex(
            `idempotency:${key}`,
            86400, // 24 hours TTL
            JSON.stringify(result)
        );
    }

    private async moveToDLQ(job: Job<TransactionJobData, TransactionResult>, error: Error): Promise<void> {
        console.log(`Moving job ${job.id} to Dead Letter Queue`);

        await this.dlq.add(
            `dlq-${job.id}`,
            job.data,
            {
                priority: 1,
                removeOnComplete: false,
                removeOnFail: false,
            }
        );

        // Alert administrators (could integrate with email/Slack/PagerDuty)
        await this.alertAdministrators(job, error);
    }

    private async alertAdministrators(job: Job<TransactionJobData, TransactionResult>, error: Error): Promise<void> {
        // TODO: Integrate with notification service (email, Slack, PagerDuty)
        console.error(`🚨 ADMIN ALERT: Transaction job ${job.id} moved to DLQ`);
        console.error(`Idempotency Key: ${job.data.idempotencyKey}`);
        console.error(`Error: ${error.message}`);
        console.error(`Job Data:`, JSON.stringify(job.data, null, 2));
    }

    /**
     * Add a transaction to the queue
     */
    async enqueueTransaction(data: TransactionJobData): Promise<string> {
        const job = await this.queue.add(`tx-${data.idempotencyKey}`, data, {
            jobId: data.idempotencyKey, // Use idempotency key as job ID to prevent duplicates
        });

        console.log(`Enqueued transaction job ${job.id}`);
        return job.id!;
    }

    /**
     * Get transaction status by idempotency key
     */
    async getTransactionStatus(idempotencyKey: string): Promise<{
        status: TransactionStatus;
        result?: TransactionResult;
        job?: any;
    }> {
        const job = await this.queue.getJob(idempotencyKey);

        if (!job) {
            // Check idempotency cache
            const cachedResult = await this.checkIdempotency(idempotencyKey);
            if (cachedResult) {
                return {
                    status: cachedResult.success ? TransactionStatus.COMPLETED : TransactionStatus.FAILED,
                    result: cachedResult,
                };
            }

            return { status: TransactionStatus.PENDING };
        }

        const state = await job.getState();
        const result = job.returnvalue;

        // Map BullMQ states to our TransactionStatus enum
        let status: TransactionStatus;
        switch (state) {
            case 'completed':
                status = TransactionStatus.COMPLETED;
                break;
            case 'failed':
                status = TransactionStatus.FAILED;
                break;
            case 'active':
                status = TransactionStatus.PROCESSING;
                break;
            case 'waiting':
                status = TransactionStatus.PENDING;
                break;
            default:
                status = TransactionStatus.PENDING;
        }

        return {
            status,
            result,
            job: {
                id: job.id,
                attemptsMade: job.attemptsMade,
                processedOn: job.processedOn,
                finishedOn: job.finishedOn,
                failedReason: job.failedReason,
            },
        };
    }

    /**
     * Get DLQ jobs for admin review
     */
    async getDLQJobs(limit: number = 50): Promise<Job<TransactionJobData, TransactionResult>[]> {
        return await this.dlq.getJobs(["completed", "failed", "waiting"], 0, limit - 1);
    }

    /**
     * Retry a job from DLQ
     */
    async retryDLQJob(jobId: string): Promise<void> {
        const job = await this.dlq.getJob(jobId);
        if (!job) {
            throw new Error(`Job ${jobId} not found in DLQ`);
        }

        // Re-enqueue to main queue
        await this.enqueueTransaction(job.data);

        // Remove from DLQ
        await job.remove();

        console.log(`Retried job ${jobId} from DLQ`);
    }

    /**
     * Graceful shutdown
     */
    async close(): Promise<void> {
        await this.worker.close();
        await this.queue.close();
        await this.dlq.close();
        await this.queueEvents.close();
        await this.connection.quit();
        console.log("Transaction queue service closed");
    }
}

export default new TransactionQueueService();

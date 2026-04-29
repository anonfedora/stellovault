import { ConflictError, NotFoundError, ValidationError } from "../config/errors";
import { prisma } from "./database.service";

const UUID_REGEX = /^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/;

interface CreateCollateralRequest {
    escrowId: string;
    assetCode?: string;
    amount: string | number;
    metadataHash: string;
}

interface TokenizeAssetRequest {
    userId: string;
    assetData: {
        assetType: string;
        assetCode?: string;
        amount: string | number;
        description?: string;
        issuer?: string;
    };
    documents?: Array<{ name: string; hash: string; mimeType?: string }>;
    escrowId: string;
}

interface CollateralListQuery {
    escrowId?: string;
    status?: string;
    page?: string | number;
    limit?: string | number;
}

const COLLATERAL_STATUSES = new Set(["LOCKED", "RELEASED", "LIQUIDATED"]);
const SUPPORTED_TYPES = ["INVOICE", "COMMODITY", "RECEIVABLE", "INVENTORY", "REAL_ESTATE"];
const DEFAULT_LTV_RATIOS: Record<string, number> = {
    INVOICE: 0.8,
    COMMODITY: 0.7,
    RECEIVABLE: 0.75,
    INVENTORY: 0.6,
    REAL_ESTATE: 0.65,
};

const MIN_PAGE = 1;
const DEFAULT_PAGE = 1;
const DEFAULT_LIMIT = 20;
const MAX_LIMIT = 100;

function parsePositiveAmount(value: string | number | undefined, fieldName: string): number {
    const amount = Number(value);
    if (!Number.isFinite(amount) || amount <= 0) {
        throw new ValidationError(`${fieldName} must be a positive number`);
    }
    return amount;
}

function normalizeStatus(value: string | undefined, fieldName: string): string {
    const status = value?.trim().toUpperCase();
    if (!status || !COLLATERAL_STATUSES.has(status)) {
        throw new ValidationError(`${fieldName} must be one of: LOCKED, RELEASED, LIQUIDATED`);
    }
    return status;
}

function coercePage(value: string | number | undefined): number {
    const page = Number(value ?? DEFAULT_PAGE);
    if (!Number.isFinite(page) || page < MIN_PAGE) return DEFAULT_PAGE;
    return Math.floor(page);
}

function coerceLimit(value: string | number | undefined): number {
    const limit = Number(value ?? DEFAULT_LIMIT);
    if (!Number.isFinite(limit) || limit < 1) return DEFAULT_LIMIT;
    return Math.min(MAX_LIMIT, Math.floor(limit));
}

export class CollateralService {
    private indexerTimer: NodeJS.Timeout | null = null;
    private polling = false;

    async tokenizeAsset(input: TokenizeAssetRequest) {
        const { userId, assetData, documents = [], escrowId } = input;

        if (!userId?.trim()) throw new ValidationError("userId is required");
        if (!escrowId?.trim()) throw new ValidationError("escrowId is required");
        if (!assetData?.assetType?.trim()) throw new ValidationError("assetType is required");

        const amount = parsePositiveAmount(assetData.amount, "amount");
        const assetType = assetData.assetType.trim().toUpperCase();

        if (!SUPPORTED_TYPES.includes(assetType)) {
            throw new ValidationError(`assetType must be one of: ${SUPPORTED_TYPES.join(", ")}`);
        }

        const escrow = await prisma.escrow.findUnique({ where: { id: escrowId }, select: { id: true } });
        if (!escrow) throw new ValidationError("escrowId does not exist");

        const metadataHash = Buffer.from(
            JSON.stringify({ userId, assetType, amount, documents, ts: Date.now() })
        ).toString("base64");

        try {
            const collateral = await prisma.collateral.create({
                data: {
                    escrowId,
                    amount: amount.toString(),
                    assetCode: assetData.assetCode || assetType,
                    metadataHash,
                    status: "LOCKED",
                },
            });
            return {
                ...collateral,
                assetType,
                documents,
                description: assetData.description,
                issuer: assetData.issuer,
                stellarAssetCode: `${assetType.slice(0, 4)}${collateral.id.slice(0, 4).toUpperCase()}`,
            };
        } catch (error: unknown) {
            if (
                typeof error === "object" && error !== null &&
                "code" in error && (error as { code: string }).code === "P2002"
            ) {
                throw new ConflictError("Collateral with this metadata already exists");
            }
            throw error;
        }
    }

    async updateValuation(collateralId: string, newUSDValue: number) {
        if (!UUID_REGEX.test(collateralId?.trim() ?? "")) {
            throw new ValidationError("Invalid collateral ID format");
        }
        if (!Number.isFinite(newUSDValue) || newUSDValue <= 0) {
            throw new ValidationError("newUSDValue must be a positive number");
        }

        const collateral = await prisma.collateral.findUnique({ where: { id: collateralId } });
        if (!collateral) throw new NotFoundError("Collateral not found");

        const updated = await prisma.collateral.update({
            where: { id: collateralId },
            data: { amount: newUSDValue.toString(), updatedAt: new Date() },
        });

        return { ...updated, usdValue: newUSDValue, updatedAt: updated.updatedAt };
    }

    async verifyCollateral(collateralId: string, verificationData: { verifier: string; method: string; result: string }) {
        if (!UUID_REGEX.test(collateralId?.trim() ?? "")) {
            throw new ValidationError("Invalid collateral ID format");
        }
        if (!verificationData?.verifier?.trim()) throw new ValidationError("verifier is required");
        if (!verificationData?.method?.trim()) throw new ValidationError("method is required");

        const collateral = await prisma.collateral.findUnique({ where: { id: collateralId } });
        if (!collateral) throw new NotFoundError("Collateral not found");

        const verificationHash = Buffer.from(
            JSON.stringify({ collateralId, ...verificationData, ts: Date.now() })
        ).toString("base64");

        const updated = await prisma.collateral.update({
            where: { id: collateralId },
            data: { metadataHash: verificationHash, updatedAt: new Date() },
        });

        return {
            ...updated,
            verified: verificationData.result === "PASS",
            verificationData,
            verifiedAt: new Date(),
        };
    }

    async getCollateralMetadata(collateralId: string) {
        if (!UUID_REGEX.test(collateralId?.trim() ?? "")) {
            throw new ValidationError("Invalid collateral ID format");
        }

        const collateral = await prisma.collateral.findUnique({
            where: { id: collateralId },
            include: { escrow: { select: { id: true, status: true, amount: true, assetCode: true } } },
        });

        if (!collateral) throw new NotFoundError("Collateral not found");

        const ltv = await this.calculateLTV(collateralId);

        return {
            ...collateral,
            ltv,
            stellarAssetCode: `${collateral.assetCode.slice(0, 4)}${collateral.id.slice(0, 4).toUpperCase()}`,
        };
    }

    async calculateLTV(collateralId: string) {
        if (!UUID_REGEX.test(collateralId?.trim() ?? "")) {
            throw new ValidationError("Invalid collateral ID format");
        }

        const collateral = await prisma.collateral.findUnique({
            where: { id: collateralId },
            include: { loans: { where: { status: { in: ["PENDING", "ACTIVE"] } }, select: { amount: true } } },
        });

        if (!collateral) throw new NotFoundError("Collateral not found");

        const collateralValue = Number(collateral.amount);
        const totalLoanAmount = collateral.loans.reduce((sum, l) => sum + Number(l.amount), 0);
        const assetType = collateral.assetCode.toUpperCase();
        const maxLtvRatio = DEFAULT_LTV_RATIOS[assetType] ?? 0.7;
        const currentLtv = collateralValue > 0 ? totalLoanAmount / collateralValue : 0;
        const maxLoanAmount = collateralValue * maxLtvRatio;

        return {
            collateralId,
            collateralValue,
            totalLoanAmount,
            currentLtv: Math.round(currentLtv * 10000) / 100,
            maxLtvRatio: maxLtvRatio * 100,
            maxLoanAmount,
            availableCredit: Math.max(0, maxLoanAmount - totalLoanAmount),
        };
    }

    async createCollateral(payload: CreateCollateralRequest) {
        const escrowId = payload.escrowId?.trim();
        const metadataHash = payload.metadataHash?.trim();

        if (!escrowId) throw new ValidationError("escrowId is required");
        if (!metadataHash) throw new ValidationError("metadataHash is required");

        const amount = parsePositiveAmount(payload.amount, "amount");

        const escrow = await prisma.escrow.findUnique({ where: { id: escrowId }, select: { id: true } });
        if (!escrow) throw new ValidationError("escrowId does not exist");

        try {
            return await prisma.collateral.create({
                data: {
                    escrowId,
                    amount: amount.toString(),
                    assetCode: payload.assetCode || "USDC",
                    metadataHash,
                    status: "LOCKED",
                },
            });
        } catch (error: unknown) {
            if (
                typeof error === "object" && error !== null &&
                "code" in error && (error as { code: string }).code === "P2002"
            ) {
                throw new ConflictError("Collateral with this metadataHash already exists");
            }
            throw error;
        }
    }

    async getCollateralById(id: string) {
        const trimmedId = id?.trim();
        if (!trimmedId || !UUID_REGEX.test(trimmedId)) {
            throw new ValidationError("Invalid collateral ID format");
        }

        const collateral = await prisma.collateral.findUnique({ where: { id: trimmedId } });
        if (!collateral) throw new NotFoundError("Collateral not found");
        return collateral;
    }

    async getCollateralByMetadataHash(hash: string) {
        const trimmedHash = hash?.trim();
        if (!trimmedHash) throw new ValidationError("Metadata hash is required");

        const collateral = await prisma.collateral.findFirst({ where: { metadataHash: trimmedHash } });
        if (!collateral) throw new NotFoundError("Collateral not found");
        return collateral;
    }

    async listCollateral(query: CollateralListQuery) {
        const page = coercePage(query.page);
        const limit = coerceLimit(query.limit);
        const skip = (page - 1) * limit;

        const where: Record<string, string> = {};
        if (query.escrowId?.trim()) where.escrowId = query.escrowId.trim();
        if (query.status?.trim()) where.status = normalizeStatus(query.status, "status");

        const [items, total] = await Promise.all([
            prisma.collateral.findMany({ where, orderBy: { createdAt: "desc" }, skip, take: limit }),
            prisma.collateral.count({ where }),
        ]);

        return {
            items,
            pagination: { page, limit, total, totalPages: Math.max(1, Math.ceil(total / limit)) },
        };
    }

    getCollateralTypes() {
        return SUPPORTED_TYPES.map((type) => ({
            type,
            maxLtvRatio: (DEFAULT_LTV_RATIOS[type] ?? 0.7) * 100,
            description: {
                INVOICE: "Trade invoices and receivables",
                COMMODITY: "Physical commodities (metals, agricultural goods)",
                RECEIVABLE: "Accounts receivable",
                INVENTORY: "Warehouse inventory",
                REAL_ESTATE: "Real estate assets",
            }[type] ?? type,
        }));
    }

    startIndexer() {
        if (this.indexerTimer) return;
        console.log("Starting collateral indexer...");
        this.indexerTimer = setInterval(async () => {
            if (this.polling) return;
            this.polling = true;
            try {
                await prisma.collateral.findMany({
                    where: { status: "LOCKED" },
                    select: { id: true, metadataHash: true },
                });
            } catch (error) {
                console.error("Collateral indexer polling failed:", error);
            } finally {
                this.polling = false;
            }
        }, 30_000);
        this.indexerTimer.unref();
    }

    stopIndexer() {
        if (this.indexerTimer) {
            clearInterval(this.indexerTimer);
            this.indexerTimer = null;
            this.polling = false;
            console.log("Collateral indexer stopped.");
        }
    }
}

export default new CollateralService();

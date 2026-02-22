import WebSocket from "ws";
import { EventEmitter } from "events";

interface ExtendedWebSocket extends WebSocket {
    isAlive?: boolean;
}

export interface EscrowCreatedPayload {
    type: "ESCROW_CREATED";
    escrowId: string;
    buyerId: string;
    sellerId: string;
}

export interface EscrowUpdatedPayload {
    type: "ESCROW_UPDATED";
    escrowId: string;
    status: string;
}

export interface LoanUpdatedPayload {
    type: "LOAN_UPDATED";
    loanId: string;
    status: string;
}

export interface GovernanceVoteCastPayload {
    type: "GOVERNANCE_VOTE_CAST";
    proposalId: string;
    newTally: number;
}

export type WebSocketEvent = 
    | EscrowCreatedPayload
    | EscrowUpdatedPayload
    | LoanUpdatedPayload
    | GovernanceVoteCastPayload;

interface ClientConnection {
    ws: WebSocket;
    isAlive: boolean;
    lastPing: Date;
}

export class WsState {
    private connections = new Set<ExtendedWebSocket>();
    private pingInterval: NodeJS.Timeout;

    constructor() {
        this.pingInterval = setInterval(() => {
            this.connections.forEach((ws) => {
                if (!ws.isAlive) {
                    ws.terminate();
                    this.connections.delete(ws);
                    return;
                }
                ws.isAlive = false;
                ws.ping();
            });
        }, 30000);
    }

    addConnection(ws: WebSocket): void {
        const extendedWs = ws as ExtendedWebSocket;
        this.connections.add(extendedWs);
        extendedWs.isAlive = true;
        
        extendedWs.on('pong', () => {
            extendedWs.isAlive = true;
        });

        extendedWs.on('close', () => {
            this.connections.delete(extendedWs);
        });

        extendedWs.on('error', () => {
            this.connections.delete(extendedWs);
        });
    }

    broadcastEvent(event: WebSocketEvent): void {
        const message = JSON.stringify(event);
        this.connections.forEach((ws) => {
            if (ws.readyState === WebSocket.OPEN) {
                ws.send(message);
            }
        });
    }

    getConnectionCount(): number {
        return this.connections.size;
    }

    cleanup(): void {
        if (this.pingInterval) {
            clearInterval(this.pingInterval);
        }
        this.connections.forEach((ws) => {
            ws.close();
        });
        this.connections.clear();
    }
}

export class WebSocketService {
    private wsState: WsState;

    constructor() {
        this.wsState = new WsState();
    }

    getWsState(): WsState {
        return this.wsState;
    }

    broadcastEscrowCreated(escrowId: string, buyerId: string, sellerId: string): void {
        const payload: EscrowCreatedPayload = {
            type: "ESCROW_CREATED",
            escrowId,
            buyerId,
            sellerId,
        };
        this.wsState.broadcastEvent(payload);
    }

    broadcastEscrowUpdated(escrowId: string, status: string): void {
        const payload: EscrowUpdatedPayload = {
            type: "ESCROW_UPDATED",
            escrowId,
            status,
        };
        this.wsState.broadcastEvent(payload);
    }

    broadcastLoanUpdated(loanId: string, status: string): void {
        const payload: LoanUpdatedPayload = {
            type: "LOAN_UPDATED",
            loanId,
            status,
        };
        this.wsState.broadcastEvent(payload);
    }

    broadcastGovernanceVoteCast(proposalId: string, newTally: number): void {
        const payload: GovernanceVoteCastPayload = {
            type: "GOVERNANCE_VOTE_CAST",
            proposalId,
            newTally,
        };
        this.wsState.broadcastEvent(payload);
    }
}

export default new WebSocketService();

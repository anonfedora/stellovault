import { EventEmitter } from "events";

export interface EscrowUpdatedPayload {
    type: "ESCROW_UPDATED";
    escrowId: string;
    status: string;
}

/**
 * In-process broadcast hub for escrow updates.
 * A real WebSocket server can subscribe to these events and fan out to clients.
 */
export class WebSocketService {
    private bus = new EventEmitter();

    broadcastEscrowUpdated(escrowId: string, status: string): void {
        const payload: EscrowUpdatedPayload = {
            type: "ESCROW_UPDATED",
            escrowId,
            status,
        };
        this.bus.emit("ESCROW_UPDATED", payload);
    }

    onEscrowUpdated(listener: (payload: EscrowUpdatedPayload) => void): () => void {
        this.bus.on("ESCROW_UPDATED", listener);
        return () => this.bus.off("ESCROW_UPDATED", listener);
    }
}

export default new WebSocketService();

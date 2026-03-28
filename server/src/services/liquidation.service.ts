// import cron from 'node-cron'; // Install: npm i node-cron @types/node-cron
import { prisma } from './database.service';
import contractService from './contract.service';
import blockchainService from './blockchain.service';
import websocketService from './websocket.service';
import { xdr } from '@stellar/stellar-sdk';
import { contracts } from '../config/contracts';
import { env } from '../config/env';
import Decimal from 'decimal.js';

const LTV_THRESHOLD = new Decimal('0.85'); // 85%
const CHECK_INTERVAL_MINUTES = 10;
const LIQUIDATION_RETRY_DELAY_MS = 5000;
const MAX_RETRIES = 3;

interface LiquidationCheck {
  loanId: string;
  borrowerId: string;
  lenderId: string;
  loanAmount: Decimal;
  collateralValue: Decimal;
  currentLTV: Decimal;
  assetCode: string;
}

export class LiquidationService {
  private cronJob: any = null; // cron.ScheduledTask
  private isRunning = false;

  async start() {
    if (this.isRunning) {
      console.log('Liquidation worker already running');
      return;
    }

    // this.cronJob = cron.schedule(`*/${CHECK_INTERVAL_MINUTES} * * * *`, async () => {
      await this.performLiquidationChecks();
    });

    console.log(`Liquidation worker started - checks every ${CHECK_INTERVAL_MINUTES} minutes`);
    this.isRunning = true;
  }

  async stop() {
    if (this.cronJob) {
      this.cronJob.stop();
      this.cronJob = null;
    }
    this.isRunning = false;
    console.log('Liquidation worker stopped');
  }

  private async performLiquidationChecks() {
    try {
      console.log('Starting liquidation checks...');

      // Fetch active loans with collateral
      const activeLoans = await prisma.loan.findMany({
        where: {
          status: 'ACTIVE',
        },
        include: {
          borrower: true,
          lender: true,
          collateral: true,
        },
      });

      if (activeLoans.length === 0) {
        console.log('No active loans to check');
        return;
      }

      console.log(`Checking ${activeLoans.length} active loans`);

      const liquidationChecks: LiquidationCheck[] = [];

      for (const loan of activeLoans) {
        try {
          const loanAmount = new Decimal(loan.amount);
          const collateralWallet = loan.collateral.walletAddress || '';
          
          if (!collateralWallet) {
            console.warn(`Loan ${loan.id} missing collateral wallet address`);
            continue;
          }

          // Fetch real-time collateral value from Stellar DEX/account balance
          // Assuming collateral asset code stored in collateral.assetCode
          const collateralAssetCode = loan.collateral.assetCode || loan.assetCode || 'USDC';
          const collateralValueStr = await blockchainService.getAccountBalance(
            collateralWallet,
            collateralAssetCode,
          );

          const collateralValue = new Decimal(collateralValueStr || '0');

          const currentLTV = loanAmount.div(collateralValue).abs();

          liquidationChecks.push({
            loanId: loan.id,
            borrowerId: loan.borrowerId,
            lenderId: loan.lenderId,
            loanAmount,
            collateralValue,
            currentLTV,
            assetCode: collateralAssetCode,
          });

          if (currentLTV.gt(LTV_THRESHOLD)) {
            console.log(`🚨 LIQUIDATION TRIGGERED: Loan ${loan.id} LTV=${currentLTV.toFixed(4)} > ${LTV_THRESHOLD.toFixed(2)}`);
            await this.triggerLiquidation(loan.id, loan.borrowerId, loan.lenderId);
          }
        } catch (error) {
          console.error(`Error checking loan ${loan.id}:`, error);
        }
      }

      // Log summary
      const highRisk = liquidationChecks.filter(check => check.currentLTV.gt(LTV_THRESHOLD));
      console.log(`Liquidation summary: ${highRisk.length}/${liquidationChecks.length} loans over threshold`);

    } catch (error) {
      console.error('Liquidation check failed:', error);
    }
  }

  private async triggerLiquidation(loanId: string, borrowerId: string, lenderId: string, retryCount = 0): Promise<void> {
    try {
  const escrowManagerId = contracts.escrow || '';
      if (!escrowManagerId) {
        throw new Error('ESCROW_MANAGER_CONTRACT_ID not configured');
      }

      // Build liquidation invoke (assuming EscrowManager has liquidate_loan method)
      // Params: loan_id (as bytes), borrower, lender
      const loanIdBytes = xdr.ScVal.scvBytes(new TextEncoder().encode(loanId));
      const xdrResult = await contractService.buildContractInvokeXDR(
        escrowManagerId,
        'refund_escrow', // Triggers liquidation by refunding expired/high-risk escrow
        [
          loanIdBytes,
          xdr.ScVal.scvAddress(xdr.PublicKey.publicKeyTypeEd25519(new TextEncoder().encode(borrowerId))),
          xdr.ScVal.scvAddress(xdr.PublicKey.publicKeyTypeEd25519(new TextEncoder().encode(lenderId))),
        ],
        env.feePayer.publicKey,
      );

      // Submit signed XDR (backend signs as fee payer)
      const submitResult = await contractService.submitXDR(xdrResult);

      console.log(`✅ Liquidation successful for loan ${loanId}: ${submitResult.hash}`);

      // Update loan status in DB
      await prisma.loan.update({
        where: { id: loanId },
        data: { status: 'DEFAULTED' },
      });

      // Notify borrower and lender
      websocketService.broadcastLoanUpdated(loanId, 'LIQUIDATED');
      
    } catch (error: any) {
      console.error(`Liquidation failed for loan ${loanId}:`, error);

      if (retryCount < MAX_RETRIES) {
        console.log(`Retrying liquidation for loan ${loanId} in ${LIQUIDATION_RETRY_DELAY_MS / 1000}s...`);
        setTimeout(() => {
          this.triggerLiquidation(loanId, borrowerId, lenderId, retryCount + 1);
        }, LIQUIDATION_RETRY_DELAY_MS);
      } else {
        console.error(`Max retries exceeded for loan ${loanId}`);
        // Still notify
        websocketService.broadcastLoanUpdated(loanId, 'LIQUIDATION_FAILED');
      }
    }
  }
}

export default new LiquidationService();


-- Create webhook auth method enum
CREATE TYPE "WebhookAuthMethod" AS ENUM (
  'SIGNATURE',
  'BEARER',
  'BASIC',
  'CUSTOM_HEADER',
  'NONE'
);

-- Create webhook delivery status enum
CREATE TYPE "WebhookDeliveryStatus" AS ENUM (
  'PENDING',
  'RETRYING',
  'SUCCESS',
  'FAILED'
);

-- Create webhooks table
CREATE TABLE "Webhook" (
  "id" TEXT NOT NULL,
  "userId" TEXT NOT NULL,
  "url" TEXT NOT NULL,
  "secret" TEXT NOT NULL,
  "events" TEXT[] DEFAULT ARRAY[]::TEXT[],
  "isActive" BOOLEAN NOT NULL DEFAULT true,
  "authMethod" "WebhookAuthMethod" NOT NULL DEFAULT 'SIGNATURE',
  "authConfig" JSONB,
  "encryptionEnabled" BOOLEAN NOT NULL DEFAULT false,
  "encryptionKey" TEXT,
  "rateLimitPerMinute" INTEGER NOT NULL DEFAULT 60,
  "lastSuccessAt" TIMESTAMP(3),
  "lastFailureAt" TIMESTAMP(3),
  "consecutiveFailures" INTEGER NOT NULL DEFAULT 0,
  "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "updatedAt" TIMESTAMP(3) NOT NULL,
  CONSTRAINT "Webhook_pkey" PRIMARY KEY ("id")
);

-- Create webhook deliveries table
CREATE TABLE "WebhookDelivery" (
  "id" TEXT NOT NULL,
  "webhookId" TEXT NOT NULL,
  "eventType" TEXT NOT NULL,
  "status" "WebhookDeliveryStatus" NOT NULL DEFAULT 'PENDING',
  "attempt" INTEGER NOT NULL DEFAULT 0,
  "requestHeaders" JSONB,
  "requestBody" JSONB,
  "responseStatus" INTEGER,
  "responseBody" TEXT,
  "errorMessage" TEXT,
  "nextRetryAt" TIMESTAMP(3),
  "deliveredAt" TIMESTAMP(3),
  "durationMs" INTEGER,
  "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "updatedAt" TIMESTAMP(3) NOT NULL,
  CONSTRAINT "WebhookDelivery_pkey" PRIMARY KEY ("id")
);

-- Add foreign keys
ALTER TABLE "Webhook"
ADD CONSTRAINT "Webhook_userId_fkey"
FOREIGN KEY ("userId") REFERENCES "User"("id")
ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE "WebhookDelivery"
ADD CONSTRAINT "WebhookDelivery_webhookId_fkey"
FOREIGN KEY ("webhookId") REFERENCES "Webhook"("id")
ON DELETE CASCADE ON UPDATE CASCADE;

-- Indexes
CREATE INDEX "Webhook_userId_idx" ON "Webhook"("userId");
CREATE INDEX "Webhook_isActive_idx" ON "Webhook"("isActive");
CREATE INDEX "Webhook_createdAt_idx" ON "Webhook"("createdAt");

CREATE INDEX "WebhookDelivery_webhookId_createdAt_idx" ON "WebhookDelivery"("webhookId", "createdAt");
CREATE INDEX "WebhookDelivery_status_nextRetryAt_idx" ON "WebhookDelivery"("status", "nextRetryAt");
CREATE INDEX "WebhookDelivery_eventType_idx" ON "WebhookDelivery"("eventType");

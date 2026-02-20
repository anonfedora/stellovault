import { defineConfig } from "prisma/config";

// DATABASE_URL must be set in the environment (e.g. from .env).
// From server dir you can run: node -r dotenv/config node_modules/.bin/prisma migrate dev
export default defineConfig({
  schema: "prisma/schema.prisma",
  migrations: {
    path: "prisma/migrations",
  },
  datasource: {
    url: process.env.DATABASE_URL ?? "",
  },
});

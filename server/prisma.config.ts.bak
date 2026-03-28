import "dotenv/config";
import { defineConfig, env } from "prisma/config"; // Import 'env' helper

export default defineConfig({
  schema: "prisma/schema.prisma",

  datasource: {
    // Use the env() helper for better type safety in v7
    url: env("DATABASE_URL"), 
  },
});


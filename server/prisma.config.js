"use strict";
// /// <reference types="node" />
// import "dotenv/config";
// import { defineConfig } from "prisma/config";
Object.defineProperty(exports, "__esModule", { value: true });
// export default defineConfig({
//   schema: "prisma/schema.prisma",
//   datasource: {
//     url: process.env.DATABASE_URL!,
//   },
// });
require("dotenv/config");
const config_1 = require("prisma/config"); // Import 'env' helper
exports.default = (0, config_1.defineConfig)({
    schema: "prisma/schema.prisma",
    datasource: {
        // Use the env() helper for better type safety in v7
        url: (0, config_1.env)("DATABASE_URL"),
    },
});

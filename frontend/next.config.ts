import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  reactStrictMode: true,
  /* Enable SWC minification for faster builds and smaller bundles */
  // swcMinify: true, // Next.js 13+ enables this by default, but keeping it explicit if needed.
};

export default nextConfig;

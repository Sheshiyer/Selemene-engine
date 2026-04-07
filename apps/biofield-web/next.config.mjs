/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "standalone",
  reactStrictMode: true,
  transpilePackages: ["@selemene/biofield-domain", "@selemene/biofield-api-client"],
};

export default nextConfig;

/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  // Workspace packages whose source is raw .ts/.tsx need Next to
  // transpile them. @selemene/dyad-ui ships TS source via main/types.
  transpilePackages: ["@selemene/dyad-ui"],
};

export default nextConfig;

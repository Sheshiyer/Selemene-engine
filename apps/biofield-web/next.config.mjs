/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  // Transpile local monorepo packages (source-only, no dist).
  transpilePackages: [
    "@selemene/biofield-domain",
    "@selemene/biofield-api-client",
  ],
  async redirects() {
    return [
      {
        source: "/",
        destination: "/login",
        permanent: false,
      },
    ];
  },
};

export default nextConfig;

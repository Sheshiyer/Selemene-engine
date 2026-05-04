/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  // Transpile local monorepo packages (source-only, no dist).
  // @selemene/noesis-sdk-ts is resolved via tsconfig paths → local src.
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

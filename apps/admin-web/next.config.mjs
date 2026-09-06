/** @type {import('next').NextConfig} */
const nextConfig = {
  basePath: "/admin",
  // Vercel's adapter owns packaging; standalone remains available for self-hosting.
  // https://github.com/vercel/next.js/issues/96646
  output: process.env.VERCEL ? undefined : "standalone",
  reactStrictMode: true,
  async redirects() {
    return [
      {
        source: "/",
        destination: "/admin/login",
        permanent: false,
        basePath: false
      },
      {
        source: "/favicon.ico",
        destination: "/admin/favicon.svg",
        permanent: false,
        basePath: false
      }
    ];
  }
};

export default nextConfig;

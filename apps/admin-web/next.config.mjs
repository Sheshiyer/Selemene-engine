/** @type {import('next').NextConfig} */
const nextConfig = {
  basePath: "/admin",
  output: "standalone",
  reactStrictMode: true,
  async redirects() {
    return [
      {
        source: "/",
        destination: "/admin/login",
        permanent: false,
        basePath: false
      }
    ];
  }
};

export default nextConfig;

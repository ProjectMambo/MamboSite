/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',
  basePath: '/MamboSite', // Ensure this matches your GitHub repo name
  images: {
    unoptimized: true,
  },
};

export default nextConfig;
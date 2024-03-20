/** @type {import('next').NextConfig} */
const nextConfig = {
    output: "export",
    experimental: {
        missingSuspenseWithCSRBailout: false,
    },
    trailingSlash: true
};

export default nextConfig;

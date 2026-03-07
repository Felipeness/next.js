import type { NextConfig } from 'next'

const nextConfig: NextConfig = {
  cacheComponents: true,
  productionBrowserSourceMaps: true,
  experimental: {
    // With `prerenderEarlyExit`, we get "Export encountered an error on ..., exiting the build."
    // and for some reason it's logged with a flaky timing.
    // This doesn't slow us down because we're prerendering one page at at time anyway.
    prerenderEarlyExit: false,
  },
  typescript: {
    ignoreBuildErrors: true,
  },
}

export default nextConfig

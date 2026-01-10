import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright E2E test configuration for Choice Sherpa frontend.
 *
 * Tests require:
 * - Frontend dev server running on localhost:5173
 * - Zitadel running on localhost:8085
 * - Test user: admin@localhost / RootPassword1!
 */
export default defineConfig({
	testDir: './tests/e2e',
	fullyParallel: false, // Auth tests need sequential execution
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 2 : 0,
	workers: 1, // Single worker for auth state consistency
	reporter: 'html',

	// Global timeout for each test
	timeout: 60000,

	use: {
		baseURL: 'http://localhost:5173',
		trace: 'on-first-retry',
		screenshot: 'only-on-failure',
		// Increase action timeout for OIDC redirects
		actionTimeout: 15000,
		navigationTimeout: 30000
	},

	projects: [
		{
			name: 'chromium',
			use: { ...devices['Desktop Chrome'] }
		}
	],

	// Start dev server before tests
	webServer: {
		command: 'npm run dev',
		url: 'http://localhost:5173',
		reuseExistingServer: !process.env.CI,
		timeout: 120000
	}
});

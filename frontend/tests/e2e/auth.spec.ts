import { test, expect, type Page } from '@playwright/test';

/**
 * E2E Authentication Tests for Choice Sherpa
 *
 * Prerequisites:
 * - Zitadel running on localhost:8085 (docker-compose up)
 * - Frontend dev server running (npm run dev)
 *
 * Test Categories:
 * 1. Smoke Tests - Verify OIDC integration without actual login (always pass)
 * 2. Login Flow Tests - Require a working test user in Zitadel
 *
 * Test User Setup:
 * Create a test user in Zitadel Console with:
 * - Username: testuser@localhost
 * - Password: TestPassword1!
 * - MFA: Disabled (Login Settings > MFA Settings > Force MFA = off)
 * - Password Change: Not required
 */

// Test credentials (dedicated test user without 2FA)
const TEST_USER = {
	username: 'testuser@localhost',
	password: 'TestPassword2!'
};

/**
 * Helper: Complete Zitadel login flow
 *
 * Uses the Auth.js signIn() function triggered by button click.
 * Includes retry logic for headless browser reliability.
 */
async function loginWithZitadel(page: Page, username: string, password: string): Promise<void> {
	// Navigate to login page
	await page.goto('/login');

	// Wait for the button to be ready and click it
	const signInButton = page.getByRole('button', { name: /sign in with zitadel/i });
	await expect(signInButton).toBeVisible();

	// Click and wait for redirect to Zitadel with retry
	let redirected = false;
	for (let attempt = 0; attempt < 3 && !redirected; attempt++) {
		if (attempt > 0) {
			// Reload and try again
			await page.goto('/login');
			await expect(signInButton).toBeVisible();
		}

		// Use click with navigation wait
		await signInButton.click();

		try {
			await page.waitForURL(/localhost:8085/, { timeout: 10000 });
			redirected = true;
		} catch {
			// Redirect didn't happen, retry
		}
	}

	if (!redirected) {
		throw new Error('Failed to redirect to Zitadel after 3 attempts');
	}

	// Enter username
	await page.getByLabel('Login Name').fill(username);
	await page.getByRole('button', { name: 'Next' }).click();

	// Enter password
	await page.waitForSelector('input[type="password"]');
	await page.getByLabel('Password').fill(password);
	await page.getByRole('button', { name: 'Next' }).click();

	// Handle password change screen if present
	const changePassword = page.getByText('Change Password');
	if (await changePassword.isVisible({ timeout: 2000 }).catch(() => false)) {
		await page.getByLabel('Old Password').fill(password);
		await page.getByLabel('New Password').fill(password);
		await page.getByLabel('Password confirmation').fill(password);
		await page.getByRole('button', { name: 'Next' }).click();
	}

	// Handle optional 2FA setup screen - skip if present
	const mfaSetup = page.getByText('2-Factor Setup');
	if (await mfaSetup.isVisible({ timeout: 2000 }).catch(() => false)) {
		// Look for skip button or cancel
		const skipButton = page.getByRole('button', { name: /skip|cancel/i });
		if (await skipButton.isVisible({ timeout: 1000 }).catch(() => false)) {
			await skipButton.click();
		}
	}

	// Wait for redirect back to app
	await page.waitForURL(/localhost:5173/, { timeout: 15000 });
}

// ============================================================================
// Smoke Tests - Verify OIDC Integration (No Login Required)
// ============================================================================

test.describe('OIDC Integration Smoke Tests', () => {
	test('should display sign-in button on home page', async ({ page }) => {
		await page.goto('/');

		// Verify sign-in button is visible
		await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
	});

	test('should redirect to Zitadel on sign-in click', async ({ page }) => {
		// Navigate to login page
		await page.goto('/login');

		// Wait for the button to be ready
		const signInButton = page.getByRole('button', { name: /sign in with zitadel/i });
		await expect(signInButton).toBeVisible();

		// Click and wait for redirect with retry logic
		let redirected = false;
		for (let attempt = 0; attempt < 3 && !redirected; attempt++) {
			if (attempt > 0) {
				await page.goto('/login');
				await expect(signInButton).toBeVisible();
			}

			await signInButton.click();

			try {
				await page.waitForURL(/localhost:8085/, { timeout: 10000 });
				redirected = true;
			} catch {
				// Retry
			}
		}

		expect(redirected).toBe(true);

		// Verify we're on the Zitadel login page
		await expect(page.getByLabel('Login Name')).toBeVisible();
	});

	test('should redirect to login page when accessing protected route', async ({ page }) => {
		// Navigate to protected route
		await page.goto('/sessions');

		// Should be redirected to login page
		await page.waitForURL(/\/login/, { timeout: 5000 });

		// The login page should have a Zitadel sign-in button
		const signInButton = page.getByRole('button', { name: 'Sign in with Zitadel' });
		await expect(signInButton).toBeVisible();
	});

	test('should have working login page at /login', async ({ page }) => {
		await page.goto('/login');

		// Verify login page content
		await expect(page.getByText('Welcome to Choice Sherpa')).toBeVisible();
		await expect(page.getByRole('button', { name: /sign in with zitadel/i })).toBeVisible();
	});
});

// ============================================================================
// Login Flow Tests - Require Working Test User
// These tests are skipped by default. To run them:
// 1. Set up the test user in Zitadel (see file header)
// 2. Run: npx playwright test --grep "Login Flow"
// ============================================================================

// Skip these tests unless TEST_USER_CONFIGURED is set
const loginTestsEnabled = process.env.TEST_USER_CONFIGURED === 'true';

test.describe('Login Flow Tests', () => {
	test.skip(!loginTestsEnabled, 'Requires configured test user - set TEST_USER_CONFIGURED=true');

	test('should complete full login and logout cycle', async ({ page }) => {
		// Navigate to app
		await page.goto('/');

		// Verify not logged in
		await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();

		// Perform login
		await loginWithZitadel(page, TEST_USER.username, TEST_USER.password);

		// Verify logged in state - Sign out button visible confirms successful login
		await expect(page.getByRole('button', { name: 'Sign out' })).toBeVisible();

		// Perform logout
		await page.getByRole('button', { name: 'Sign out' }).click();

		// Verify logged out
		await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
	});

	test('should persist session across page reloads', async ({ page }) => {
		await page.goto('/');
		await loginWithZitadel(page, TEST_USER.username, TEST_USER.password);

		// Verify logged in
		await expect(page.getByRole('button', { name: 'Sign out' })).toBeVisible();

		// Reload page
		await page.reload();

		// Should still be logged in
		await expect(page.getByRole('button', { name: 'Sign out' })).toBeVisible();
	});

	test('should redirect to requested page after login', async ({ page }) => {
		// Navigate to login page with callback to sessions page
		await page.goto('/login?callbackUrl=%2Fsessions');

		// Wait for the button to be ready
		const signInButton = page.getByRole('button', { name: /sign in with zitadel/i });
		await expect(signInButton).toBeVisible();

		// Click and wait for redirect with retry logic
		let redirected = false;
		for (let attempt = 0; attempt < 3 && !redirected; attempt++) {
			if (attempt > 0) {
				await page.goto('/login?callbackUrl=%2Fsessions');
				await expect(signInButton).toBeVisible();
			}

			await signInButton.click();

			try {
				await page.waitForURL(/localhost:8085/, { timeout: 10000 });
				redirected = true;
			} catch {
				// Retry
			}
		}

		expect(redirected).toBe(true);

		// Complete login
		await page.getByLabel('Login Name').fill(TEST_USER.username);
		await page.getByRole('button', { name: 'Next' }).click();
		await page.waitForSelector('input[type="password"]');
		await page.getByLabel('Password').fill(TEST_USER.password);
		await page.getByRole('button', { name: 'Next' }).click();

		// Should be redirected to sessions page after login
		await page.waitForURL(/sessions/, { timeout: 20000 });
	});
});

// ============================================================================
// Token Expiration Tests - Require Working Test User
// ============================================================================

test.describe('Token Expiration Handling', () => {
	test.skip(!loginTestsEnabled, 'Requires configured test user - set TEST_USER_CONFIGURED=true');

	test('should handle expired session cookie gracefully', async ({ page, context }) => {
		// Login first
		await page.goto('/');
		await loginWithZitadel(page, TEST_USER.username, TEST_USER.password);
		await expect(page.getByRole('button', { name: 'Sign out' })).toBeVisible();

		// Clear session cookies to simulate expiration
		await context.clearCookies();

		// Reload page
		await page.reload();

		// Should show sign in button (session cleared)
		await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
	});

	test('should allow re-authentication after session expires', async ({ page, context }) => {
		// Login
		await page.goto('/');
		await loginWithZitadel(page, TEST_USER.username, TEST_USER.password);

		// Clear cookies to simulate expiration
		await context.clearCookies();
		await page.reload();

		// Verify logged out
		await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();

		// Should be able to login again
		await loginWithZitadel(page, TEST_USER.username, TEST_USER.password);
		await expect(page.getByRole('button', { name: 'Sign out' })).toBeVisible();
	});

	test('should maintain valid session within expiry window', async ({ page }) => {
		await page.goto('/');
		await loginWithZitadel(page, TEST_USER.username, TEST_USER.password);

		// Wait a few seconds (within expiry window)
		await page.waitForTimeout(3000);

		// Navigate around
		await page.goto('/');

		// Should still be logged in
		await expect(page.getByRole('button', { name: 'Sign out' })).toBeVisible();
	});
});

// ============================================================================
// Multi-Device Session Tests - Require Working Test User
// ============================================================================

test.describe('Multi-Device Session Management', () => {
	test.skip(!loginTestsEnabled, 'Requires configured test user - set TEST_USER_CONFIGURED=true');

	test('should allow concurrent sessions from different browsers', async ({ browser }) => {
		// Create two separate browser contexts (simulating different devices)
		const context1 = await browser.newContext();
		const context2 = await browser.newContext();

		const page1 = await context1.newPage();
		const page2 = await context2.newPage();

		try {
			// Login on "device 1"
			await page1.goto('http://localhost:5173');
			await loginWithZitadel(page1, TEST_USER.username, TEST_USER.password);
			await expect(page1.getByRole('button', { name: 'Sign out' })).toBeVisible();

			// Login on "device 2"
			await page2.goto('http://localhost:5173');
			await loginWithZitadel(page2, TEST_USER.username, TEST_USER.password);
			await expect(page2.getByRole('button', { name: 'Sign out' })).toBeVisible();

			// Both sessions should be active
			await page1.reload();
			await expect(page1.getByRole('button', { name: 'Sign out' })).toBeVisible();

			await page2.reload();
			await expect(page2.getByRole('button', { name: 'Sign out' })).toBeVisible();
		} finally {
			await context1.close();
			await context2.close();
		}
	});

	test('should logout independently on each device', async ({ browser }) => {
		const context1 = await browser.newContext();
		const context2 = await browser.newContext();

		const page1 = await context1.newPage();
		const page2 = await context2.newPage();

		try {
			// Login on both devices
			await page1.goto('http://localhost:5173');
			await loginWithZitadel(page1, TEST_USER.username, TEST_USER.password);

			await page2.goto('http://localhost:5173');
			await loginWithZitadel(page2, TEST_USER.username, TEST_USER.password);

			// Logout on device 1
			await page1.getByRole('button', { name: 'Sign out' }).click();
			await expect(page1.getByRole('button', { name: 'Sign in' })).toBeVisible();

			// Device 2 should still be logged in
			await page2.reload();
			await expect(page2.getByRole('button', { name: 'Sign out' })).toBeVisible();
		} finally {
			await context1.close();
			await context2.close();
		}
	});

	test('should handle rapid login/logout cycles', async ({ page }) => {
		await page.goto('/');

		// Perform multiple login/logout cycles
		for (let i = 0; i < 3; i++) {
			await loginWithZitadel(page, TEST_USER.username, TEST_USER.password);
			await expect(page.getByRole('button', { name: 'Sign out' })).toBeVisible();

			await page.getByRole('button', { name: 'Sign out' }).click();
			await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
		}
	});
});

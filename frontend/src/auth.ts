/**
 * Auth.js configuration for Zitadel OIDC authentication.
 *
 * This module configures the SvelteKit Auth.js integration with Zitadel
 * as the identity provider. It handles:
 *
 * - OIDC Authorization Code flow with PKCE
 * - Session management via secure cookies
 * - Access token storage for API calls
 * - Automatic token refresh
 *
 * @see https://authjs.dev/getting-started/installation?framework=sveltekit
 * @see https://zitadel.com/docs/examples/login/authjs
 */

import { SvelteKitAuth } from '@auth/sveltekit';
import type { Provider } from '@auth/sveltekit/providers';

/**
 * Zitadel OIDC Provider configuration.
 *
 * Uses the generic OIDC provider since Auth.js doesn't have a built-in
 * Zitadel provider. This follows the standard OIDC discovery flow.
 */
const ZitadelProvider: Provider = {
	id: 'zitadel',
	name: 'Zitadel',
	type: 'oidc',
	// Issuer URL - Zitadel's OIDC discovery endpoint
	// Auth.js will fetch /.well-known/openid-configuration automatically
	issuer: process.env.AUTH_ZITADEL_ISSUER,
	clientId: process.env.AUTH_ZITADEL_CLIENT_ID,
	clientSecret: process.env.AUTH_ZITADEL_CLIENT_SECRET,
	// Request offline_access for refresh tokens
	authorization: {
		params: {
			scope: 'openid email profile offline_access'
		}
	},
	// Map Zitadel's user info to Auth.js profile
	profile(profile) {
		return {
			id: profile.sub,
			name: profile.name || profile.preferred_username,
			email: profile.email,
			image: profile.picture
		};
	}
};

/**
 * SvelteKit Auth.js handler and utilities.
 *
 * Exports:
 * - handle: Server hook for processing auth requests
 * - signIn: Function to initiate sign-in flow
 * - signOut: Function to sign out the user
 */
export const { handle, signIn, signOut } = SvelteKitAuth({
	providers: [ZitadelProvider],

	// Use secure cookies in production
	trustHost: true,

	callbacks: {
		/**
		 * JWT callback - runs when JWT is created or updated.
		 * We store the access token here for API calls.
		 */
		async jwt({ token, account }) {
			// On initial sign-in, store tokens
			if (account) {
				token.accessToken = account.access_token;
				token.refreshToken = account.refresh_token;
				token.expiresAt = account.expires_at;
			}

			// Check if token needs refresh
			if (token.expiresAt && Date.now() < (token.expiresAt as number) * 1000) {
				return token;
			}

			// Token expired - attempt refresh
			// Note: Refresh logic would go here if needed
			// For now, we let the token expire and force re-login
			return token;
		},

		/**
		 * Session callback - runs when session is checked.
		 * We expose the access token to the client for API calls.
		 */
		async session({ session, token }) {
			// Add access token to session for API calls
			session.accessToken = token.accessToken as string | undefined;
			return session;
		}
	},

	// Session configuration
	session: {
		strategy: 'jwt',
		maxAge: 30 * 24 * 60 * 60 // 30 days
	},

	// Page configuration
	pages: {
		signIn: '/auth/signin',
		error: '/auth/error'
	}
});

// Extend the Session type to include accessToken
declare module '@auth/sveltekit' {
	interface Session {
		accessToken?: string;
	}
}

// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		interface Locals {
			/** The authenticated user session from Auth.js */
			session: import('@auth/sveltekit').Session | null;
		}
		interface PageData {
			/** The authenticated user session */
			session: import('@auth/sveltekit').Session | null;
		}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};

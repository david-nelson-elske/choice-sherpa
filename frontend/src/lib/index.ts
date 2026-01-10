/**
 * Library exports for Choice Sherpa frontend.
 *
 * Re-exports commonly used utilities and types.
 */

// API client utilities
export {
	authFetch,
	authGet,
	authPost,
	authPut,
	authDelete,
	AuthenticationError,
	ApiError,
	type AuthFetchOptions
} from './api/client';

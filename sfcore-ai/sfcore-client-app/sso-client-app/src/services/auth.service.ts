import { api, ApiResponse, TokenStorage, redirectToLogin } from './api';
import { 
  LoginRequest, 
  RegisterData, 
  LoginResponse, 
  User, 
  UserPermissions 
} from '@/types/auth.types';

// ============================================================================
// Auth Service
// ============================================================================

export const authService = {
  /**
   * Login with email and password
   */
  async login(data: LoginRequest): Promise<LoginResponse> {
    const response = await api.post<ApiResponse<LoginResponse>>('/auth/login', data);
    const { access_token, refresh_token, user } = response.data.data;
    
    TokenStorage.setTokens(access_token, refresh_token);
    return { user, access_token, refresh_token };
  },

  /**
   * Register new user
   */
  async register(data: RegisterData): Promise<User> {
    const response = await api.post<ApiResponse<{ user: User }>>('/auth/register', data);
    return response.data.data.user;
  },

  /**
   * Get current authenticated user
   */
  async getCurrentUser(): Promise<User> {
    const response = await api.get<ApiResponse<User>>('/auth/me');
    return response.data.data;
  },

  /**
   * Logout - clear tokens and redirect
   */
  logout() {
    // Optional: call logout endpoint to invalidate tokens on server
    api.post('/auth/logout').catch(() => {});
    redirectToLogin();
  },

  /**
   * Refresh access token
   */
  async refreshToken(): Promise<string> {
    const refreshToken = TokenStorage.getRefreshToken();
    if (!refreshToken) throw new Error('No refresh token');

    const response = await api.post<ApiResponse<{ access_token: string; refresh_token: string }>>(
      '/auth/refresh',
      { refresh_token: refreshToken }
    );

    const { access_token, refresh_token } = response.data.data;
    TokenStorage.setTokens(access_token, refresh_token);
    return access_token;
  },

  /**
   * Check if user is authenticated
   */
  isAuthenticated(): boolean {
    return TokenStorage.isAuthenticated();
  },

  /**
   * Verify email
   */
  async verifyEmail(token: string): Promise<void> {
    await api.post('/auth/verify-email', { token });
  },

  /**
   * Request password reset
   */
  async forgotPassword(email: string): Promise<void> {
    await api.post('/auth/forgot-password', { email });
  },

  /**
   * Reset password with token
   */
  async resetPassword(token: string, newPassword: string): Promise<void> {
    await api.post('/auth/reset-password', { token, password: newPassword });
  },

  /**
   * Get user permissions for a specific client app
   */
  async getUserPermissions(clientAppId: string): Promise<UserPermissions> {
    const response = await api.get<ApiResponse<UserPermissions>>(
      `/users/permissions?client_app_id=${clientAppId}`
    );
    return response.data.data;
  },
};

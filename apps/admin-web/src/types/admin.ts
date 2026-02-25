export interface LoginResponse {
  token: string;
  user_id: string;
  email: string;
  tier: string;
}

export interface AdminSession {
  user_id: string;
  email: string;
  tier: string;
  permissions: string[];
  roles: string[];
  has_admin_access: boolean;
}

export interface ApiErrorPayload {
  error: string;
  error_code: string;
  details?: Record<string, unknown> | null;
}

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

export interface AdminUserItem {
  id: string;
  email: string;
  full_name: string;
  tier: string;
  consciousness_level: number;
  experience_points: number;
  last_login_at: string | null;
  failed_login_attempts: number;
  locked_until: string | null;
  state: "active" | "locked" | string;
  active_key_count: number;
  permissions: string[];
  created_at: string;
  updated_at: string;
}

export interface AdminUsersResponse {
  items: AdminUserItem[];
  total: number;
  limit: number;
  offset: number;
}

export interface UpdateUserStateResponse {
  user_id: string;
  state: string;
  locked_until: string | null;
  message: string;
}

export interface UpdateUserTierResponse {
  user_id: string;
  tier: string;
  message: string;
}

export interface UpdateUserRolesResponse {
  user_id: string;
  roles: string[];
  permissions: string[];
  affected_api_keys: number;
  message: string;
}

export interface AdminApiKeyItem {
  id: string;
  user_id: string;
  user_email: string;
  tier: string;
  permissions: string[];
  consciousness_level: number;
  rate_limit: number;
  created_at: string;
  expires_at: string | null;
  last_used: string | null;
  is_active: boolean;
}

export interface AdminApiKeysResponse {
  items: AdminApiKeyItem[];
  total: number;
  limit: number;
  offset: number;
}

export interface CreateApiKeyResponse {
  key: AdminApiKeyItem;
  secret_key: string;
}

export interface RotateApiKeyResponse {
  old_key_id: string;
  key: AdminApiKeyItem;
  secret_key: string;
}

export interface AdminHistorySyncUserItem {
  user_id: string;
  email: string;
  readings_count: number;
  usage_events_count: number;
  drift_count: number;
  status: string;
  last_reading_at: string | null;
  last_event_at: string | null;
}

export interface AdminHistorySyncUsersResponse {
  items: AdminHistorySyncUserItem[];
  total: number;
  limit: number;
  offset: number;
}

export interface AdminHistorySyncDeviceItem {
  device_id: string;
  user_id: string;
  user_email: string;
  tier: string;
  status: string;
  permission_count: number;
  last_seen_at: string | null;
  created_at: string;
}

export interface AdminHistorySyncDevicesResponse {
  items: AdminHistorySyncDeviceItem[];
  total: number;
  limit: number;
  offset: number;
}

export interface AdminHistorySyncEventItem {
  event_id: string;
  user_id: string;
  user_email: string;
  engine_id: string | null;
  workflow_id: string | null;
  status: string;
  duration_ms: number;
  occurred_at: string;
}

export interface AdminHistorySyncEventsResponse {
  items: AdminHistorySyncEventItem[];
  total: number;
  limit: number;
  offset: number;
}

export interface AdminAnalyticsSummaryResponse {
  window_hours: number;
  requests_total: number;
  success_total: number;
  failure_total: number;
  error_rate_pct: number;
  p95_duration_ms: number;
  avg_duration_ms: number;
  active_users: number;
  unique_keys: number;
}

export interface AdminAnalyticsTimeseriesPoint {
  bucket_start: string;
  request_count: number;
  success_count: number;
  failure_count: number;
  avg_duration_ms: number;
}

export interface AdminAnalyticsTimeseriesResponse {
  window_hours: number;
  bucket: string;
  points: AdminAnalyticsTimeseriesPoint[];
}

export interface AdminAnalyticsBreakdownEntry {
  label: string;
  request_count: number;
}

export interface AdminAnalyticsBreakdownResponse {
  window_hours: number;
  engines: AdminAnalyticsBreakdownEntry[];
  workflows: AdminAnalyticsBreakdownEntry[];
}

export interface AdminAnalyticsTopConsumerItem {
  user_id: string;
  user_email: string;
  request_count: number;
  failure_count: number;
  avg_duration_ms: number;
}

export interface AdminAnalyticsTopConsumersResponse {
  window_hours: number;
  items: AdminAnalyticsTopConsumerItem[];
}

export interface ApiErrorPayload {
  error: string;
  error_code: string;
  details?: Record<string, unknown> | null;
}

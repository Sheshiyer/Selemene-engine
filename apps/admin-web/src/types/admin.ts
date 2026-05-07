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
  name: string | null;
  key_prefix: string | null;
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

export interface AdminUsageWindowSummary {
  total: number;
  success: number;
  failure: number;
  active_users: number;
}

export interface AdminUsageDailyPoint {
  day: string;
  request_count: number;
}

export interface AdminUsageEngineEntry {
  engine_id: string;
  request_count: number;
}

export interface AdminUsageTierEntry {
  tier: string;
  request_count: number;
}

export interface AdminUsageTopUserEntry {
  user_id: string;
  user_email: string;
  request_count: number;
}

export interface AdminUsageSummaryResponse {
  daily: AdminUsageWindowSummary;
  monthly: AdminUsageWindowSummary;
  daily_requests: AdminUsageDailyPoint[];
  engine_breakdown: AdminUsageEngineEntry[];
  tier_distribution: AdminUsageTierEntry[];
  top_users: AdminUsageTopUserEntry[];
}

export interface AdminSystemSubsystemStatus {
  name: string;
  status: string;
  detail: string;
  latency_ms: number | null;
}

export interface AdminSystemHealthResponse {
  checked_at: string;
  overall_status: string;
  uptime_seconds: number;
  subsystems: AdminSystemSubsystemStatus[];
}

export interface AdminSystemServiceItem {
  id: string;
  name: string;
  category: string;
  status: string;
  detail: string;
  latency_ms: number | null;
  error_rate_pct: number | null;
  updated_at: string;
}

export interface AdminSystemServicesResponse {
  items: AdminSystemServiceItem[];
  total: number;
  limit: number;
  offset: number;
}

export interface AdminSystemWorkflowItem {
  workflow_id: string;
  name: string;
  engine_count: number;
  recent_runs: number;
  failure_runs: number;
  last_seen_at: string | null;
  status: string;
}

export interface AdminSystemWorkflowsResponse {
  window_hours: number;
  items: AdminSystemWorkflowItem[];
  total: number;
  limit: number;
  offset: number;
}

export interface AdminSystemCacheResponse {
  checked_at: string;
  redis_available: boolean;
  l1_entries: number;
  total_requests: number;
  l1_hits: number;
  l2_hits: number;
  l3_hits: number;
  cache_misses: number;
  hit_rate_pct: number;
}

export interface AdminAuditEventItem {
  event_id: string;
  request_id: string;
  occurred_at: string;
  actor_user_id: string;
  actor_email: string;
  action: string;
  target_type: string;
  target_id: string | null;
  result: string;
  duration_ms: number;
  metadata: Record<string, unknown>;
}

export interface AdminAuditEventsResponse {
  items: AdminAuditEventItem[];
  total: number;
  limit: number;
  offset: number;
}

export interface AdminAuditEventDetailResponse {
  event: AdminAuditEventItem;
}

export interface AdminAuditActionsResponse {
  actions: string[];
}

export interface ApiErrorPayload {
  error: string;
  error_code: string;
  details?: Record<string, unknown> | null;
}

// ─── Billing (Dodo Payments) ─────────────────────────────────────────────────

export interface AdminBillingStatusCount {
  status: string;
  count: number;
}

export interface AdminBillingOverviewResponse {
  status_counts: AdminBillingStatusCount[];
  free_users: number;
  mrr_usd_estimate: number;
}

export interface AdminBillingSubscriptionItem {
  id: string;
  user_id: string;
  plan_id: string;
  provider: string;
  provider_customer_id: string | null;
  provider_subscription_id: string | null;
  status: string;
  cancel_at_period_end: boolean;
  current_period_start: string | null;
  current_period_end: string | null;
  canceled_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface AdminBillingSubscriptionsResponse {
  items: AdminBillingSubscriptionItem[];
  total: number;
  limit: number;
  offset: number;
}

export interface AdminBillingSubscriptionDetailResponse {
  subscription: AdminBillingSubscriptionItem;
}

export interface AdminBillingWebhookEventItem {
  webhook_id: string;
  provider: string;
  event_type: string;
  processed_at: string;
}

export interface AdminBillingWebhookEventsResponse {
  items: AdminBillingWebhookEventItem[];
  limit: number;
}

export interface AdminBillingReconcileRun {
  id: number;
  started_at: string;
  finished_at: string | null;
  force_cancel: boolean;
  drift_json: Record<string, unknown>;
  error: string | null;
}

export interface AdminBillingReconcileDriftResponse {
  latest: AdminBillingReconcileRun | null;
}

export interface AdminBillingReconcileTriggerResponse {
  command: string;
  note: string;
}

export interface AdminBillingPlanItem {
  id: string;
  code: string;
  display_name: string;
  description: string | null;
  is_active: boolean;
  dodo_product_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface AdminBillingPlansResponse {
  items: AdminBillingPlanItem[];
}

export interface AdminBillingCancelSubscriptionResponse {
  ok: boolean;
  subscription_id: string;
}
